use crate::prelude::*;

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
    Object(Option<Ident>),
    NewId(Option<Ident>),
    Array,
    Fd,
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
            Ty::NewId(id) => id.as_ref(),
            _ => None,
        }
    }
}

impl Parse for Arg {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        fn opt_iface(parser: &mut Parser) -> Result<Option<Ident>, Error> {
            let Some(_) = parser.next_punct_of('<') else {
                return Ok(None);
            };
            let wl_name = parser.parse::<Ident>()?;
            let name = wl_name.to_camel();
            parser.punct_of('>')?;
            Ok(Some(name))
        }
        fn opt_enum(is_signed: bool, parser: &mut Parser) -> Result<Ty, Error> {
            let Some(_) = parser.next_punct_of('<') else {
                return Ok(if is_signed { Ty::Int } else { Ty::Uint });
            };
            let wl_name = parser.parse::<Ident>()?;
            let path = match parser.next_punct_of('.') {
                Some(_) => {
                    let wl_subname = parser.parse::<Ident>()?;
                    let name = wl_subname.to_camel();
                    Path(Some(wl_name), name)
                },
                None => {
                    let name = wl_name.to_camel();
                    Path(None, name)
                },
            };
            parser.punct_of('>')?;
            Ok(Ty::Enum { is_signed, path })
        }

        let name = parser.parse::<Ident>()?;
        parser.punct_of(':')?;
        let ty = parser.parse::<Ident>()?;
        let ty = match ty.as_str() {
            "int" => opt_enum(true, parser)?,
            "uint" => opt_enum(false, parser)?,
            "fixed" => Ty::Fixed,
            "string" => Ty::String,
            "object" => Ty::Object(opt_iface(parser)?),
            "new_id" => Ty::NewId(opt_iface(parser)?),
            "array" => Ty::Array,
            "fd" => Ty::Fd,
            "version" => Ty::Version,
            _ => return Err(Error::new("unknown type", ty)),
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
            Ty::Object(None) => id!(Object),
            Ty::Object(Some(i)) => gr!(Object<#i>),
            Ty::NewId(None) => id!(NewId),
            Ty::NewId(Some(i)) => gr!(NewId<#i>),
            Ty::Array => gr!(&'a [u8]),
            Ty::Fd => id!(RawFd),
            Ty::Version => id!(Version),
        }
    }
}

