extern crate alloc;

use alloc::borrow::Cow;
use core::ops;
use repository::{self, EntityPtr, Repo};
use thiserror::Error;

mod parse;

pub struct InfoSet<'input> {
    input: Cow<'input, str>,
    repo: Repo,
    pub doc_info_item: EntityPtr<DocInfoItem>,
}

impl<'input> ops::Deref for InfoSet<'input> {
    type Target = Repo;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

pub struct InfoSetStatistics {
    pub has_nonstandard_entity_reference: bool,
}

#[derive(Copy, Clone)]
pub enum DocChildInfoItemPtr {
    Element(ElementInfoItemPtr),
    PI(PIInfoItemPtr),
    Comment(CommentInfoItemPtr),
    DTD(DTDInfoItemPtr),
}

#[derive(Copy, Clone)]
pub enum ElementChildInfoItemPtr {
    Element(ElementInfoItemPtr),
    PI(PIInfoItemPtr),
    UER(UERInfoItemPtr),
    CharGroup(CharGroupInfoItemPtr),
    Comment(CommentInfoItemPtr),
}

#[derive(Copy, Clone)]
pub enum ElementParentInfoItemPtr {
    Doc(DocInfoItemPtr),
    Element(ElementInfoItemPtr),
}

#[derive(Copy, Clone)]
pub enum AttrReferenceInfoItemPtr {
    Element(ElementInfoItemPtr),
    UE(UEInfoItemPtr),
    Notation(NotationInfoItemPtr),
}

#[derive(Copy, Clone)]
pub enum PIParentInfoItemPtr {
    Doc(DocInfoItemPtr),
    Element(ElementInfoItemPtr),
    DTD(DTDInfoItemPtr),
}

#[derive(Copy, Clone)]
pub enum CommentParentInfoItemPtr {
    Doc(DocInfoItemPtr),
    Element(ElementInfoItemPtr),
}

pub type DocInfoItemPtr = EntityPtr<DocInfoItem>;
pub type ElementInfoItemPtr = EntityPtr<ElementInfoItem>;
pub type AttributeInfoItemPtr = EntityPtr<AttributeInfoItem>;
pub type PIInfoItemPtr = EntityPtr<PIInfoItem>;
pub type UERInfoItemPtr = EntityPtr<UERInfoItem>;
pub type CharGroupInfoItemPtr = EntityPtr<CharGroupInfoItem>;
pub type CommentInfoItemPtr = EntityPtr<CommentInfoItem>;
pub type DTDInfoItemPtr = EntityPtr<DTDInfoItem>;
pub type UEInfoItemPtr = EntityPtr<UEInfoItem>;
pub type NotationInfoItemPtr = EntityPtr<NotationInfoItem>;
pub type NSInfoItemPtr = EntityPtr<NSInfoItem>;

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

pub struct UriSpan(Span);

pub enum Never {}

pub enum Version {
    Version1_0,
    Version1_1,
}

pub enum EncodingScheme {
    Utf8,
    Other(Span),
}

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

pub enum UnknownOr<T> {
    Unknown,
    Known(T),
}

pub struct DocInfoItem {
    pub children: Vec<DocChildInfoItemPtr>,
    pub document_element: ElementInfoItemPtr,
    pub notations: Option<Vec<NotationInfoItemPtr>>,
    pub unparsed_entities: Vec<UEInfoItemPtr>,
    pub base_uri: UriSpan,
    pub character_encoding_scheme: EncodingScheme,
    pub standalone: Option<bool>,
    pub version: Option<Version>,
    pub all_declarations_processed: bool,
}

pub struct ElementInfoItem {
    pub namespace_name: Option<Span>,
    pub local_name: Span,
    pub prefix: Option<Span>,
    pub children: Vec<ElementChildInfoItemPtr>,
    pub attributes: Vec<AttributeInfoItemPtr>,
    pub namespace_attributes: Vec<AttributeInfoItemPtr>,
    pub in_scope_namespaces: Vec<NSInfoItemPtr>,
    pub base_uri: UriSpan,
    pub parent: ElementParentInfoItemPtr,
}

pub enum AttrSpecified {
    Specified,
    DefaultedFromDTD,
}

pub enum AttrTypeAndReferences {
    ID(Option<Never>),
    IDREF(Option<Vec<AttrReferenceInfoItemPtr>>),
    IDREFS,
    ENTITY,
    ENTITIES,
    NMTOKEN(Option<Never>),
    NMTOKENS(Option<Never>),
    NOTATION,
    CDATA(Option<Never>),
    ENUMERATION(Option<Never>),
}

pub struct AttributeInfoItem {
    pub namespace_name: Option<Span>,
    pub local_name: Span,
    pub prefix: Option<Span>,
    pub normalized_value: CowSpan,
    pub specified: AttrSpecified,
    pub attribute_type_and_references: Option<UnknownOr<AttrTypeAndReferences>>,
    pub owner_element: ElementInfoItemPtr,
}

pub struct PIInfoItem {
    pub target: Span,
    pub content: Span,
    pub base_uri: UriSpan,
    pub notation: Option<UnknownOr<NotationInfoItemPtr>>,
    pub parent: PIParentInfoItemPtr,
}

pub struct UERInfoItem {
    pub name: Span,
    pub system_identifier: UnknownOr<Option<Span>>,
    pub public_identifier: UnknownOr<Option<Span>>,
    pub declaration_base_uri: UnknownOr<Option<UriSpan>>,
    pub parent: ElementInfoItemPtr,
}

pub struct CharGroupInfoItem {
    pub characters: CowSpan,
    pub element_content_whitespace: UnknownOr<Option<bool>>,
    pub parent: ElementInfoItemPtr,
}

pub struct CommentInfoItem {
    pub content: Span,
    pub parent: CommentParentInfoItemPtr,
}

pub struct DTDInfoItem {
    pub system_identifier: Option<Span>,
    pub public_identifier: Option<Span>,
    pub children: Vec<PIInfoItemPtr>,
    pub parent: DocInfoItemPtr,
}

pub struct UEInfoItem {
    pub name: Span,
    pub system_identifier: Span,
    pub public_identifier: Option<Span>,
    pub declaration_base_uri: UriSpan,
    pub notation_name: Span,
    pub notation: UnknownOr<Option<NotationInfoItemPtr>>,
}

pub struct NotationInfoItem {
    pub name: Span,
    pub system_identifier: Option<Span>,
    pub public_identifier: Option<Span>,
    pub declaration_base_uri: UriSpan,
}

pub struct NSInfoItem {
    pub prefix: Option<Span>,
    pub namespace_name: Span,
}
