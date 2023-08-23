#[derive(Debug, Clone)]
pub(crate) struct Project {
    pub(crate) name: String,
    pub(crate) lang: String,
    pub(crate) template: String,
    pub(crate) group: Option<String>,
    pub(crate) rule: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Source<T> {
    pub(crate) remote: String,
    pub(crate) url: T,
    pub(crate) user: String,
    pub(crate) password: String,
    pub(crate) branch: Option<String>,
}
