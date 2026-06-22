
#[derive(Debug)]
pub enum Surface {
    None,
    #[expect(dead_code)]
    XdgToplevel(XdgToplevel),
}

#[expect(dead_code)]
#[derive(Debug)]
pub struct XdgToplevel {
    pub title: Box<str>,
    pub app_id: Box<str>,
}
