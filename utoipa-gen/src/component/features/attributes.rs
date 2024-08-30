use std::mem;

use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::{LitStr, Token, TypePath};

use crate::component::serde::RenameRule;
use crate::component::{schema, GenericType, TypeTree};
use crate::path::parameter::{self, ParameterStyle};
use crate::schema_type::SchemaFormat;
use crate::{parse_utils, AnyValue, Array, Diagnostics};

use super::{name, Feature, Parse};
use quote::quote;

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Default(pub(crate) Option<AnyValue>);

impl Default {
    pub fn new_default_trait(struct_ident: Ident, field_ident: syn::Member) -> Self {
        Self(Some(AnyValue::new_default_trait(struct_ident, field_ident)))
    }
}

impl Parse for Default {
    fn parse(input: syn::parse::ParseStream, _: proc_macro2::Ident) -> syn::Result<Self> {
        if input.peek(syn::Token![=]) {
            parse_utils::parse_next(input, || AnyValue::parse_any(input)).map(|any| Self(Some(any)))
        } else {
            Ok(Self(None))
        }
    }
}

impl ToTokens for Default {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match &self.0 {
            Some(inner) => tokens.extend(quote! {Some(#inner)}),
            None => tokens.extend(quote! {None}),
        }
    }
}

impl From<self::Default> for Feature {
    fn from(value: self::Default) -> Self {
        Feature::Default(value)
    }
}

name!(Default = "default");

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Example(AnyValue);

impl Parse for Example {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_next(input, || AnyValue::parse_any(input)).map(Self)
    }
}

impl ToTokens for Example {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<Example> for Feature {
    fn from(value: Example) -> Self {
        Feature::Example(value)
    }
}

name!(Example = "example");

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Examples(Vec<AnyValue>);

impl Parse for Examples {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        let examples;
        syn::parenthesized!(examples in input);

        Ok(Self(
            Punctuated::<AnyValue, Token![,]>::parse_terminated_with(
                &examples,
                AnyValue::parse_any,
            )?
            .into_iter()
            .collect(),
        ))
    }
}

impl ToTokens for Examples {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if !self.0.is_empty() {
            let examples = Array::Borrowed(&self.0).to_token_stream();
            examples.to_tokens(tokens);
        }
    }
}

impl From<Examples> for Feature {
    fn from(value: Examples) -> Self {
        Feature::Examples(value)
    }
}

name!(Examples = "examples");

#[derive(Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct XmlAttr(schema::xml::XmlAttr);

impl XmlAttr {
    /// Split [`XmlAttr`] for [`GenericType::Vec`] returning tuple of [`XmlAttr`]s where first
    /// one is for a vec and second one is for object field.
    pub fn split_for_vec(
        &mut self,
        type_tree: &TypeTree,
    ) -> Result<(Option<XmlAttr>, Option<XmlAttr>), Diagnostics> {
        if matches!(type_tree.generic_type, Some(GenericType::Vec)) {
            let mut value_xml = mem::take(self);
            let vec_xml = schema::xml::XmlAttr::with_wrapped(
                mem::take(&mut value_xml.0.is_wrapped),
                mem::take(&mut value_xml.0.wrap_name),
            );

            Ok((Some(XmlAttr(vec_xml)), Some(value_xml)))
        } else {
            self.validate_xml(&self.0)?;

            Ok((None, Some(mem::take(self))))
        }
    }

    #[inline]
    fn validate_xml(&self, xml: &schema::xml::XmlAttr) -> Result<(), Diagnostics> {
        if let Some(wrapped_ident) = xml.is_wrapped.as_ref() {
            Err(Diagnostics::with_span(
                wrapped_ident.span(),
                "cannot use `wrapped` attribute in non slice field type",
            )
            .help("Try removing `wrapped` attribute or make your field `Vec`"))
        } else {
            Ok(())
        }
    }
}

impl Parse for XmlAttr {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        let xml;
        syn::parenthesized!(xml in input);
        xml.parse::<schema::xml::XmlAttr>().map(Self)
    }
}

impl ToTokens for XmlAttr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<XmlAttr> for Feature {
    fn from(value: XmlAttr) -> Self {
        Feature::XmlAttr(value)
    }
}

name!(XmlAttr = "xml");

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Format(SchemaFormat<'static>);

impl Parse for Format {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_next(input, || input.parse::<SchemaFormat>()).map(Self)
    }
}

impl ToTokens for Format {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<Format> for Feature {
    fn from(value: Format) -> Self {
        Feature::Format(value)
    }
}

name!(Format = "format");

#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct WriteOnly(bool);

impl Parse for WriteOnly {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for WriteOnly {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<WriteOnly> for Feature {
    fn from(value: WriteOnly) -> Self {
        Feature::WriteOnly(value)
    }
}

name!(WriteOnly = "write_only");

#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ReadOnly(bool);

impl Parse for ReadOnly {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for ReadOnly {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<ReadOnly> for Feature {
    fn from(value: ReadOnly) -> Self {
        Feature::ReadOnly(value)
    }
}

name!(ReadOnly = "read_only");

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Title(String);

impl Parse for Title {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_next_literal_str(input).map(Self)
    }
}

impl ToTokens for Title {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<Title> for Feature {
    fn from(value: Title) -> Self {
        Feature::Title(value)
    }
}

name!(Title = "title");

#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Nullable(bool);

impl Nullable {
    pub fn new() -> Self {
        Self(true)
    }

    pub fn value(&self) -> bool {
        self.0
    }

    pub fn into_schema_type_token_stream(self) -> proc_macro2::TokenStream {
        if self.0 {
            quote! {utoipa::openapi::schema::Type::Null}
        } else {
            proc_macro2::TokenStream::new()
        }
    }
}

impl Parse for Nullable {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for Nullable {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<Nullable> for Feature {
    fn from(value: Nullable) -> Self {
        Feature::Nullable(value)
    }
}

name!(Nullable = "nullable");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Rename(String);

impl Rename {
    pub fn into_value(self) -> String {
        self.0
    }
}

impl Parse for Rename {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_next_literal_str(input).map(Self)
    }
}

impl ToTokens for Rename {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<Rename> for Feature {
    fn from(value: Rename) -> Self {
        Feature::Rename(value)
    }
}

name!(Rename = "rename");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct RenameAll(RenameRule);

impl RenameAll {
    pub fn as_rename_rule(&self) -> &RenameRule {
        &self.0
    }
}

impl Parse for RenameAll {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        let litstr = parse_utils::parse_next(input, || input.parse::<LitStr>())?;

        litstr
            .value()
            .parse::<RenameRule>()
            .map_err(|error| syn::Error::new(litstr.span(), error.to_string()))
            .map(Self)
    }
}

impl From<RenameAll> for Feature {
    fn from(value: RenameAll) -> Self {
        Feature::RenameAll(value)
    }
}

name!(RenameAll = "rename_all");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Style(ParameterStyle);

impl From<ParameterStyle> for Style {
    fn from(style: ParameterStyle) -> Self {
        Self(style)
    }
}

impl Parse for Style {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_next(input, || input.parse::<ParameterStyle>().map(Self))
    }
}

impl ToTokens for Style {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl From<Style> for Feature {
    fn from(value: Style) -> Self {
        Feature::Style(value)
    }
}

name!(Style = "style");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct ParameterIn(parameter::ParameterIn);

impl Parse for ParameterIn {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_next(input, || input.parse::<parameter::ParameterIn>().map(Self))
    }
}

impl ToTokens for ParameterIn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<ParameterIn> for Feature {
    fn from(value: ParameterIn) -> Self {
        Feature::ParameterIn(value)
    }
}

name!(ParameterIn = "parameter_in");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct AllowReserved(bool);

impl Parse for AllowReserved {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for AllowReserved {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl From<AllowReserved> for Feature {
    fn from(value: AllowReserved) -> Self {
        Feature::AllowReserved(value)
    }
}

name!(AllowReserved = "allow_reserved");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Explode(bool);

impl Parse for Explode {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for Explode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl From<Explode> for Feature {
    fn from(value: Explode) -> Self {
        Feature::Explode(value)
    }
}

name!(Explode = "explode");

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ValueType(syn::Type);

impl ValueType {
    /// Create [`TypeTree`] from current [`syn::Type`].
    pub fn as_type_tree(&self) -> Result<TypeTree, Diagnostics> {
        TypeTree::from_type(&self.0)
    }
}

impl Parse for ValueType {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_next(input, || input.parse::<syn::Type>()).map(Self)
    }
}

impl From<ValueType> for Feature {
    fn from(value: ValueType) -> Self {
        Feature::ValueType(value)
    }
}

name!(ValueType = "value_type");

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Inline(pub(super) bool);

impl Parse for Inline {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl From<bool> for Inline {
    fn from(value: bool) -> Self {
        Inline(value)
    }
}

impl From<Inline> for Feature {
    fn from(value: Inline) -> Self {
        Feature::Inline(value)
    }
}

name!(Inline = "inline");

/// Specify names of unnamed fields with `names(...) attribute for `IntoParams` derive.
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct IntoParamsNames(Vec<String>);

impl IntoParamsNames {
    pub fn into_values(self) -> Vec<String> {
        self.0
    }
}

impl Parse for IntoParamsNames {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        Ok(Self(
            parse_utils::parse_punctuated_within_parenthesis::<LitStr>(input)?
                .iter()
                .map(LitStr::value)
                .collect(),
        ))
    }
}

impl From<IntoParamsNames> for Feature {
    fn from(value: IntoParamsNames) -> Self {
        Feature::IntoParamsNames(value)
    }
}

name!(IntoParamsNames = "names");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct SchemaWith(TypePath);

impl Parse for SchemaWith {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_next(input, || input.parse::<TypePath>().map(Self))
    }
}

impl ToTokens for SchemaWith {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let path = &self.0;
        tokens.extend(quote! {
            #path()
        })
    }
}

impl From<SchemaWith> for Feature {
    fn from(value: SchemaWith) -> Self {
        Feature::SchemaWith(value)
    }
}

name!(SchemaWith = "schema_with");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Description(parse_utils::Value);

impl Parse for Description {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        parse_utils::parse_next_literal_str_or_expr(input).map(Self)
    }
}

impl ToTokens for Description {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<String> for Description {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl From<Description> for Feature {
    fn from(value: Description) -> Self {
        Self::Description(value)
    }
}

name!(Description = "description");

/// Deprecated feature parsed from macro attributes.
///
/// This feature supports only syntax parsed from utoipa specific macro attributes, it does not
/// support Rust `#[deprecated]` attribute.
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Deprecated(bool);

impl Parse for Deprecated {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for Deprecated {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let deprecated: crate::Deprecated = self.0.into();
        deprecated.to_tokens(tokens);
    }
}

impl From<Deprecated> for Feature {
    fn from(value: Deprecated) -> Self {
        Self::Deprecated(value)
    }
}

name!(Deprecated = "deprecated");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct As(pub TypePath);

impl Parse for As {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        parse_utils::parse_next(input, || input.parse()).map(Self)
    }
}

impl From<As> for Feature {
    fn from(value: As) -> Self {
        Self::As(value)
    }
}

name!(As = "as");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct AdditionalProperties(bool);

impl Parse for AdditionalProperties {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for AdditionalProperties {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let additional_properties = &self.0;
        tokens.extend(quote!(
            utoipa::openapi::schema::AdditionalProperties::FreeForm(
                #additional_properties
            )
        ))
    }
}

name!(AdditionalProperties = "additional_properties");

impl From<AdditionalProperties> for Feature {
    fn from(value: AdditionalProperties) -> Self {
        Self::AdditionalProperties(value)
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Required(pub bool);

impl Required {
    pub fn is_true(&self) -> bool {
        self.0
    }
}

impl Parse for Required {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for Required {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl From<crate::Required> for Required {
    fn from(value: crate::Required) -> Self {
        if value == crate::Required::True {
            Self(true)
        } else {
            Self(false)
        }
    }
}

impl From<bool> for Required {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl From<Required> for Feature {
    fn from(value: Required) -> Self {
        Self::Required(value)
    }
}

name!(Required = "required");

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ContentEncoding(String);

impl Parse for ContentEncoding {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        parse_utils::parse_next_literal_str(input).map(Self)
    }
}

impl ToTokens for ContentEncoding {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

name!(ContentEncoding = "content_encoding");

impl From<ContentEncoding> for Feature {
    fn from(value: ContentEncoding) -> Self {
        Self::ContentEncoding(value)
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ContentMediaType(String);

impl Parse for ContentMediaType {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        parse_utils::parse_next_literal_str(input).map(Self)
    }
}

impl ToTokens for ContentMediaType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<ContentMediaType> for Feature {
    fn from(value: ContentMediaType) -> Self {
        Self::ContentMediaType(value)
    }
}

name!(ContentMediaType = "content_media_type");
