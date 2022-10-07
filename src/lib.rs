extern crate alloc;

use alloc::borrow::Cow;
use core::ops;
use thiserror::Error;

#[macro_use]
pub mod utils;
pub mod parse;

#[repo::repo]
#[member(
    DocInfoItem,
    ElementInfoItem,
    CharGroupInfoItem,
    AttributeInfoItem,
    PIInfoItem,
    CommentInfoItem,
    NSInfoItem,
    UEInfoItem,
    UERInfoItem,
    NotationInfoItem,
    DTDInfoItem
)]
#[derive(Default)]
pub struct InfoSetData {
    pub doc_info_item: Option<DocInfoItem>,
}

pub struct InfoSet<'input> {
    input: Cow<'input, str>,
    data: InfoSetData,
}

impl<'input> ops::Deref for InfoSet<'input> {
    type Target = InfoSetData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'input> ops::DerefMut for InfoSet<'input> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[derive(Default)]
pub struct InfoSetStatistics {
    pub has_nonstandard_entity_reference: bool,
}

#[derive(Copy, Clone)]
pub enum DocChildInfoItem {
    Element(ElementInfoItem),
    PI(PIInfoItem),
    Comment(CommentInfoItem),
    DTD(DTDInfoItem),
}

#[derive(Copy, Clone)]
pub enum ElementChildInfoItem {
    Element(ElementInfoItem),
    PI(PIInfoItem),
    UER(UERInfoItem),
    CharGroup(CharGroupInfoItem),
    Comment(CommentInfoItem),
}

impl_enum_from_variant!(ElementChildInfoItem, Element, ElementInfoItem);
impl_enum_from_variant!(ElementChildInfoItem, PI, PIInfoItem);
impl_enum_from_variant!(ElementChildInfoItem, Comment, CommentInfoItem);

#[derive(Copy, Clone)]
pub enum ElementParentInfoItem {
    Doc(DocInfoItem),
    Element(ElementInfoItem),
}

#[derive(Copy, Clone)]
pub enum AttrReferenceInfoItem {
    Element(ElementInfoItem),
    UE(UEInfoItem),
    Notation(NotationInfoItem),
}

#[derive(Copy, Clone)]
pub enum PIParentInfoItem {
    Doc(DocInfoItem),
    Element(ElementInfoItem),
    DTD(DTDInfoItem),
}

#[derive(Copy, Clone)]
pub enum CommentParentInfoItem {
    Doc(DocInfoItem),
    Element(ElementInfoItem),
}

#[derive(Clone)]
pub struct Span(ops::Range<usize>);

impl Span {
    pub fn get<'a>(&self, infoset: &'a InfoSet<'_>) -> Result<&'a str, SpanError> {
        infoset.input.get(self.0.clone()).ok_or(SpanError)
    }
}

#[derive(Debug, Error)]
#[error("Invalid string span in XML info set")]
pub struct SpanError;

#[derive(Clone)]
pub struct UriSpan(Span);

#[derive(Clone)]
pub enum Never {}

#[derive(Clone)]
pub enum Version {
    Version1_0,
    Version1_1,
    Other(String),
}

#[derive(Clone)]
pub enum EncodingScheme {
    Utf8,
    Other(String),
}

#[derive(Clone)]
pub enum CowSpan {
    Borrowed(Span),
    Owned(String),
}

impl CowSpan {
    pub fn get_ref<'a>(&'a self, infoset: &'a InfoSet<'_>) -> Result<&'a str, SpanError> {
        match self {
            CowSpan::Borrowed(span) => span.get(infoset),
            CowSpan::Owned(str) => Ok(str.as_ref()),
        }
    }
}

#[derive(Clone)]
pub enum UnknownOr<T> {
    Unknown,
    Known(T),
}

#[repo::entity(repo = InfoSetData)]
pub enum DocInfoItem {
    Parsed {
        pub version: Version,
        pub character_encoding_scheme: Option<EncodingScheme>,
        pub standalone: Option<bool>,
        pub document_element: ElementInfoItem,
        pub children: Vec<DocChildInfoItem>,
        pub notations: Option<Vec<NotationInfoItem>>,
        pub unparsed_entities: Vec<UEInfoItem>,
        pub base_uri: Option<UriSpan>,
        pub all_declarations_processed: bool,
    },
    NotYetParsed,
}

#[repo::entity(repo = InfoSetData)]
pub struct ElementInfoItem {
    pub namespace_name: Option<Span>,
    pub local_name: Span,
    pub prefix: Option<Span>,
    #[by_ref]
    pub children: Vec<ElementChildInfoItem>,
    pub attributes: Vec<AttributeInfoItem>,
    pub namespace_attributes: Vec<AttributeInfoItem>,
    pub in_scope_namespaces: Vec<NSInfoItem>,
    pub base_uri: Option<UriSpan>,
    pub parent: ElementParentInfoItem,
}

#[derive(Clone, Copy)]
pub enum AttrSpecified {
    Specified,
    DefaultedFromDTD,
}

#[derive(Clone)]
pub enum AttrTypeAndReferences {
    ID(Option<Never>),
    IDREF(Option<Vec<AttrReferenceInfoItem>>),
    IDREFS,
    ENTITY,
    ENTITIES,
    NMTOKEN(Option<Never>),
    NMTOKENS(Option<Never>),
    NOTATION,
    CDATA(Option<Never>),
    ENUMERATION(Option<Never>),
}

#[repo::entity(repo = InfoSetData)]
pub struct AttributeInfoItem {
    pub namespace_name: Option<Span>,
    pub local_name: Span,
    pub prefix: Option<Span>,
    pub normalized_value: CowSpan,
    pub specified: AttrSpecified,
    pub attribute_type_and_references: Option<UnknownOr<AttrTypeAndReferences>>,
    pub owner_element: ElementInfoItem,
}

#[repo::entity(repo = InfoSetData)]
pub struct PIInfoItem {
    pub target: Span,
    pub content: Option<Span>,
    pub base_uri: Option<UriSpan>,
    pub notation: Option<UnknownOr<NotationInfoItem>>,
    pub parent: PIParentInfoItem,
}

#[repo::entity(repo = InfoSetData)]
pub struct UERInfoItem {
    pub name: Span,
    pub system_identifier: UnknownOr<Option<Span>>,
    pub public_identifier: UnknownOr<Option<Span>>,
    pub declaration_base_uri: UnknownOr<Option<UriSpan>>,
    pub parent: ElementInfoItem,
}

#[repo::entity(repo = InfoSetData)]
pub struct CharGroupInfoItem {
    pub characters: CowSpan,
    pub element_content_whitespace: UnknownOr<Option<bool>>,
    pub parent: ElementInfoItem,
}

#[repo::entity(repo = InfoSetData)]
pub struct CommentInfoItem {
    pub content: Span,
    pub parent: CommentParentInfoItem,
}

#[repo::entity(repo = InfoSetData)]
pub struct DTDInfoItem {
    pub system_identifier: Option<Span>,
    pub public_identifier: Option<Span>,
    pub children: Vec<PIInfoItem>,
    pub parent: DocInfoItem,
}

#[repo::entity(repo = InfoSetData)]
pub struct UEInfoItem {
    pub name: Span,
    pub system_identifier: Span,
    pub public_identifier: Option<Span>,
    pub declaration_base_uri: UriSpan,
    pub notation_name: Span,
    pub notation: UnknownOr<Option<NotationInfoItem>>,
}

#[repo::entity(repo = InfoSetData)]
pub struct NotationInfoItem {
    pub name: Span,
    pub system_identifier: Option<Span>,
    pub public_identifier: Option<Span>,
    pub declaration_base_uri: UriSpan,
}

#[repo::entity(repo = InfoSetData)]
pub struct NSInfoItem {
    pub prefix: Option<Span>,
    pub namespace_name: Span,
}
