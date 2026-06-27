use crate::interface::attr;
use crate::prelude::*;

pub enum OpKind {
    Request,
    Event,
}

// ```
// #[destructor, since = 5]
// delete_id(id: uint, parent: object<wl_registry>?)
// ```
pub struct Op {
    pub wl_name: Ident,
    pub name: Ident,
    pub since: Option<Literal>,
    pub is_destructor: bool,
    pub args: Vec<Arg>,
    pub lf_ph: Option<TokenTree>,
    pub fd_idx: Option<usize>,
}

impl Parse for Op {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        let mut since = None;
        let mut is_destructor = false;

        if let Some(mut parser) = attr(parser)? {
            if parser.next_ident_of("since").is_some() {
                parser.punct_of('=')?;
                since = Some(parser.lit()?);
                parser.next_punct_of(',');
            }

            if parser.next_ident_of("destructor").is_some() {
                parser.next_punct_of(',');
                is_destructor = true;
            }

            parser.check_empty()?;
        }

        parser.parse::<Vis>()?;
        parser.ident_of("fn")?;

        let wl_name = parser.ident()?;
        let name = Ident::new_string(to_camel(wl_name.as_str()), Span::call_site());

        let mut lf_ph = None;
        let mut has_new_id = false;
        let mut fd_idx = None;

        let mut args = vec![];
        let mut arg_parser = parser.group_of(Delimiter::Parenthesis)?.body_parser();

        while arg_parser.peek().is_some() {
            let arg = arg_parser.parse::<Arg>()?;

            if arg.is_lf() {
                lf_ph = Some(Group::new(Delimiter::None, token_stream!(<'_>)).into())
            }
            if arg.ty.as_new_id().is_some() {
                if has_new_id {
                    return Err(Error::spanned("only one new_id is supported", arg.name.span()));
                }
                has_new_id = true;
            }
            if arg.ty.is_fd() {
                if fd_idx.is_some() {
                    return Err(Error::spanned("only one fd is supported", arg.name.span()));
                }
                fd_idx = Some(args.len());
            }

            args.push(arg);
            arg_parser.next_punct_of(',');
        }

        parser.punct_of(';')?;

        Ok(Self {
            wl_name,
            name,
            since,
            is_destructor,
            args,
            lf_ph,
            fd_idx,
        })
    }
}

// `id: uint, parent: object<wl_registry>?`
//
// ```
// int uint
// fixed
// string
// object<iface>
// new_id<iface>
// array
// fd
// ```
//
// `<ty>?` allow null
// `uint<iface.enum>?` enum arg
pub struct Arg {
    pub name: Ident,
    pub ty: Ty,
    #[expect(dead_code)]
    pub opt: bool,
}

impl std::ops::Deref for Arg {
    type Target = Ty;

    fn deref(&self) -> &Self::Target {
        &self.ty
    }
}

#[derive(Clone)]
pub struct Path(Option<Ident>, Ident);

impl From<Path> for TokenTree {
    fn from(value: Path) -> Self {
        Group::new(
            Delimiter::None,
            value
                .0
                .map(<_>::into)
                .into_iter()
                .chain(gentoken!(::))
                .chain(Some(TokenTree::from(value.1)))
                .collect(),
        )
        .into()
    }
}

pub enum Ty {
    Int,
    Uint,
    Enum {
        #[expect(dead_code)]
        is_signed: bool,
        path: Path,
    },
    Fixed,
    String,
    Object(Ident),
    NewId(Ident),
    Array,
    Fd,
    /// only used by wl_registry::bind
    ObjectId,
    /// only used by wl_registry::bind
    Version,
}

impl Ty {
    pub fn is_lf(&self) -> bool {
        matches!(self, Ty::String | Ty::Array)
    }

    pub fn is_fd(&self) -> bool {
        matches!(self, Ty::Fd)
    }

    pub fn as_new_id(&self) -> Option<&Ident> {
        match self {
            Ty::NewId(id) => Some(id),
            _ => None,
        }
    }
}

impl Parse for Arg {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        fn iface(parser: &mut Parser) -> Result<Ident, Error> {
            parser.punct_of('<')?;
            let wl_name = parser.ident()?;
            let name = Ident::new_string(to_camel(wl_name.as_str()), wl_name.span());
            parser.punct_of('>')?;
            Ok(name)
        }
        fn maybe_enum(is_signed: bool, parser: &mut Parser) -> Result<Ty, Error> {
            let Some(_) = parser.next_punct_of('<') else {
                return Ok(if is_signed { Ty::Int } else { Ty::Uint });
            };
            let wl_name = parser.ident()?;
            let path = match parser.next_punct_of('.') {
                Some(_) => {
                    let wl_subname = parser.ident()?;
                    let name = Ident::new_string(to_camel(wl_subname.as_str()), wl_subname.span());
                    Path(Some(wl_name), name)
                },
                None => {
                    let name = Ident::new_string(to_camel(wl_name.as_str()), wl_name.span());
                    Path(None, name)
                },
            };
            parser.punct_of('>')?;
            Ok(Ty::Enum { is_signed, path })
        }

        let name = parser.ident()?;
        parser.punct_of(':')?;
        let ty = parser.ident()?;
        let ty = match ty.as_str() {
            "int" => maybe_enum(true, parser)?,
            "uint" => maybe_enum(false, parser)?,
            "fixed" => Ty::Fixed,
            "string" => Ty::String,
            "object" => Ty::Object(iface(parser)?),
            "new_id" => Ty::NewId(iface(parser)?),
            "array" => Ty::Array,
            "fd" => Ty::Fd,
            "object_id" => Ty::ObjectId,
            "version" => Ty::Version,
            _ => return Err(Error::spanned("unknown type", ty.span())),
        };
        let opt = parser.next_punct_of('?').is_some();
        Ok(Self {
            name,
            ty,
            opt,
        })
    }
}

impl Ty {
    pub fn generate(&self) -> TokenTree {
        macro_rules! id {
            ($ty:ident) => {
                Ident::new(stringify!($ty), Span::call_site()).into()
            };
        }
        macro_rules! gr {
            ($($tt:tt)*) => {
                Group::new(Delimiter::None, token_stream!($($tt)*)).into()
            };
        }
        match self {
            Ty::Int => id!(i32),
            Ty::Uint => id!(u32),
            Ty::Enum { path, .. } => path.clone().into(),
            Ty::Fixed => id!(Fixed),
            Ty::String => gr!(&'a str),
            Ty::Object(i) => gr!(Object<#i>),
            Ty::NewId(i) => gr!(NewId<#i>),
            Ty::Array => gr!(&'a [u8]),
            Ty::Fd => id!(RawFd),
            Ty::ObjectId => id!(ObjectId),
            Ty::Version => id!(Version),
        }
    }
}
