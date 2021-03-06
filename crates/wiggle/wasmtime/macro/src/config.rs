use {
    proc_macro2::Span,
    std::collections::HashMap,
    syn::{
        braced,
        parse::{Parse, ParseStream},
        punctuated::Punctuated,
        Error, Ident, Path, Result, Token,
    },
    wiggle_generate::config::{CtxConf, WitxConf},
};

#[derive(Debug, Clone)]
pub struct Config {
    pub target: TargetConf,
    pub witx: WitxConf,
    pub ctx: CtxConf,
    pub modules: ModulesConf,
}

#[derive(Debug, Clone)]
pub enum ConfigField {
    Target(TargetConf),
    Witx(WitxConf),
    Ctx(CtxConf),
    Modules(ModulesConf),
}

mod kw {
    syn::custom_keyword!(target);
    syn::custom_keyword!(witx);
    syn::custom_keyword!(witx_literal);
    syn::custom_keyword!(ctx);
    syn::custom_keyword!(modules);
    syn::custom_keyword!(name);
    syn::custom_keyword!(docs);
    syn::custom_keyword!(function_override);
}

impl Parse for ConfigField {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::target) {
            input.parse::<kw::target>()?;
            input.parse::<Token![:]>()?;
            Ok(ConfigField::Target(input.parse()?))
        } else if lookahead.peek(kw::witx) {
            input.parse::<kw::witx>()?;
            input.parse::<Token![:]>()?;
            Ok(ConfigField::Witx(WitxConf::Paths(input.parse()?)))
        } else if lookahead.peek(kw::witx_literal) {
            input.parse::<kw::witx_literal>()?;
            input.parse::<Token![:]>()?;
            Ok(ConfigField::Witx(WitxConf::Literal(input.parse()?)))
        } else if lookahead.peek(kw::ctx) {
            input.parse::<kw::ctx>()?;
            input.parse::<Token![:]>()?;
            Ok(ConfigField::Ctx(input.parse()?))
        } else if lookahead.peek(kw::modules) {
            input.parse::<kw::modules>()?;
            input.parse::<Token![:]>()?;
            Ok(ConfigField::Modules(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Config {
    pub fn build(fields: impl Iterator<Item = ConfigField>, err_loc: Span) -> Result<Self> {
        let mut target = None;
        let mut witx = None;
        let mut ctx = None;
        let mut modules = None;
        for f in fields {
            match f {
                ConfigField::Target(c) => {
                    if target.is_some() {
                        return Err(Error::new(err_loc, "duplicate `target` field"));
                    }
                    target = Some(c);
                }
                ConfigField::Witx(c) => {
                    if witx.is_some() {
                        return Err(Error::new(err_loc, "duplicate `witx` field"));
                    }
                    witx = Some(c);
                }
                ConfigField::Ctx(c) => {
                    if ctx.is_some() {
                        return Err(Error::new(err_loc, "duplicate `ctx` field"));
                    }
                    ctx = Some(c);
                }
                ConfigField::Modules(c) => {
                    if modules.is_some() {
                        return Err(Error::new(err_loc, "duplicate `modules` field"));
                    }
                    modules = Some(c);
                }
            }
        }
        Ok(Config {
            target: target.ok_or_else(|| Error::new(err_loc, "`target` field required"))?,
            witx: witx.ok_or_else(|| Error::new(err_loc, "`witx` field required"))?,
            ctx: ctx.ok_or_else(|| Error::new(err_loc, "`ctx` field required"))?,
            modules: modules.ok_or_else(|| Error::new(err_loc, "`modules` field required"))?,
        })
    }

    /// Load the `witx` document for the configuration.
    ///
    /// # Panics
    ///
    /// This method will panic if the paths given in the `witx` field were not valid documents.
    pub fn load_document(&self) -> witx::Document {
        self.witx.load_document()
    }
}

impl Parse for Config {
    fn parse(input: ParseStream) -> Result<Self> {
        let contents;
        let _lbrace = braced!(contents in input);
        let fields: Punctuated<ConfigField, Token![,]> =
            contents.parse_terminated(ConfigField::parse)?;
        Ok(Config::build(fields.into_iter(), input.span())?)
    }
}

#[derive(Debug, Clone)]
pub struct TargetConf {
    pub path: Path,
}

impl Parse for TargetConf {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(TargetConf {
            path: input.parse()?,
        })
    }
}

enum ModuleConfField {
    Name(Ident),
    Docs(String),
    FunctionOverride(FunctionOverrideConf),
}

impl Parse for ModuleConfField {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::name) {
            input.parse::<kw::name>()?;
            input.parse::<Token![:]>()?;
            Ok(ModuleConfField::Name(input.parse()?))
        } else if lookahead.peek(kw::docs) {
            input.parse::<kw::docs>()?;
            input.parse::<Token![:]>()?;
            let docs: syn::LitStr = input.parse()?;
            Ok(ModuleConfField::Docs(docs.value()))
        } else if lookahead.peek(kw::function_override) {
            input.parse::<kw::function_override>()?;
            input.parse::<Token![:]>()?;
            Ok(ModuleConfField::FunctionOverride(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModuleConf {
    pub name: Ident,
    pub docs: Option<String>,
    pub function_override: FunctionOverrideConf,
}

impl ModuleConf {
    fn build(fields: impl Iterator<Item = ModuleConfField>, err_loc: Span) -> Result<Self> {
        let mut name = None;
        let mut docs = None;
        let mut function_override = None;
        for f in fields {
            match f {
                ModuleConfField::Name(c) => {
                    if name.is_some() {
                        return Err(Error::new(err_loc, "duplicate `name` field"));
                    }
                    name = Some(c);
                }
                ModuleConfField::Docs(c) => {
                    if docs.is_some() {
                        return Err(Error::new(err_loc, "duplicate `docs` field"));
                    }
                    docs = Some(c);
                }
                ModuleConfField::FunctionOverride(c) => {
                    if function_override.is_some() {
                        return Err(Error::new(err_loc, "duplicate `function_override` field"));
                    }
                    function_override = Some(c);
                }
            }
        }
        Ok(ModuleConf {
            name: name.ok_or_else(|| Error::new(err_loc, "`name` field required"))?,
            docs,
            function_override: function_override.unwrap_or_default(),
        })
    }
}

impl Parse for ModuleConf {
    fn parse(input: ParseStream) -> Result<Self> {
        let contents;
        let _lbrace = braced!(contents in input);
        let fields: Punctuated<ModuleConfField, Token![,]> =
            contents.parse_terminated(ModuleConfField::parse)?;
        Ok(ModuleConf::build(fields.into_iter(), input.span())?)
    }
}

#[derive(Debug, Clone)]
pub struct ModulesConf {
    pub mods: HashMap<String, ModuleConf>,
}

impl ModulesConf {
    pub fn iter(&self) -> impl Iterator<Item = (&String, &ModuleConf)> {
        self.mods.iter()
    }
}

impl Parse for ModulesConf {
    fn parse(input: ParseStream) -> Result<Self> {
        let contents;
        let _lbrace = braced!(contents in input);
        let fields: Punctuated<(String, ModuleConf), Token![,]> =
            contents.parse_terminated(|i| {
                let name = i.parse::<Ident>()?.to_string();
                i.parse::<Token![=>]>()?;
                let val = i.parse()?;
                Ok((name, val))
            })?;
        Ok(ModulesConf {
            mods: fields.into_iter().collect(),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct FunctionOverrideConf {
    pub funcs: Vec<FunctionOverrideField>,
}
impl FunctionOverrideConf {
    pub fn find(&self, name: &str) -> Option<&Ident> {
        self.funcs
            .iter()
            .find(|f| f.name == name)
            .map(|f| &f.replacement)
    }
}

impl Parse for FunctionOverrideConf {
    fn parse(input: ParseStream) -> Result<Self> {
        let contents;
        let _lbrace = braced!(contents in input);
        let fields: Punctuated<FunctionOverrideField, Token![,]> =
            contents.parse_terminated(FunctionOverrideField::parse)?;
        Ok(FunctionOverrideConf {
            funcs: fields.into_iter().collect(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FunctionOverrideField {
    pub name: String,
    pub replacement: Ident,
}
impl Parse for FunctionOverrideField {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?.to_string();
        input.parse::<Token![=>]>()?;
        let replacement = input.parse::<Ident>()?;
        Ok(FunctionOverrideField { name, replacement })
    }
}
