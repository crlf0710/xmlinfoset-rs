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
        #[hidden_accessor = version_of_parsed_doc_info_item]
        pub version: Version,
        #[hidden_accessor = character_encoding_scheme_of_parsed_doc_info_item]
        pub character_encoding_scheme: Option<EncodingScheme>,
        #[hidden_accessor = standalone_of_parsed_doc_info_item]
        pub standalone: Option<bool>,
        #[hidden_accessor = document_element_of_parsed_doc_info_item]
        pub document_element: ElementInfoItem,
        #[hidden_accessor = children_of_parsed_doc_info_item]
        #[by_ref]
        pub children: Vec<DocChildInfoItem>,
        #[hidden_accessor = notations_of_parsed_doc_info_item]
        #[by_ref]
        pub notations: Option<Vec<NotationInfoItem>>,
        #[hidden_accessor = unparsed_entities_of_parsed_doc_info_item]
        #[by_ref]
        pub unparsed_entities: Vec<UEInfoItem>,
        #[hidden_accessor = base_uri_of_parsed_doc_info_item]
        pub base_uri: Option<UriSpan>,
        #[hidden_accessor = all_declarations_processed_of_parsed_doc_info_item]
        pub all_declarations_processed: bool,
    },
    NotYetParsed,
}

impl DocInfoItem {
    pub fn version(self, __repo: &InfoSetData) -> Version {
        version_of_parsed_doc_info_item(self, __repo).unwrap()
    }

    pub fn set_version(self, __value: Version, __repo: &mut InfoSetData) {
        set_version_of_parsed_doc_info_item(self, __value, __repo).unwrap()
    }

    pub fn character_encoding_scheme(self, __repo: &InfoSetData) -> Option<EncodingScheme> {
        character_encoding_scheme_of_parsed_doc_info_item(self, __repo).unwrap()
    }

    pub fn set_character_encoding_scheme(
        self,
        __value: Option<EncodingScheme>,
        __repo: &mut InfoSetData,
    ) {
        set_character_encoding_scheme_of_parsed_doc_info_item(self, __value, __repo).unwrap()
    }

    pub fn standalone(self, __repo: &InfoSetData) -> Option<bool> {
        standalone_of_parsed_doc_info_item(self, __repo).unwrap()
    }

    pub fn set_standalone(self, __value: Option<bool>, __repo: &mut InfoSetData) {
        set_standalone_of_parsed_doc_info_item(self, __value, __repo).unwrap()
    }

    pub fn document_element(self, __repo: &InfoSetData) -> ElementInfoItem {
        document_element_of_parsed_doc_info_item(self, __repo).unwrap()
    }

    pub fn set_document_element(self, __value: ElementInfoItem, __repo: &mut InfoSetData) {
        set_document_element_of_parsed_doc_info_item(self, __value, __repo).unwrap()
    }

    pub fn children(self, __repo: &InfoSetData) -> &Vec<DocChildInfoItem> {
        children_of_parsed_doc_info_item(self, __repo).unwrap()
    }

    pub fn children_mut(self, __repo: &mut InfoSetData) -> &mut Vec<DocChildInfoItem> {
        children_of_parsed_doc_info_item_mut(self, __repo).unwrap()
    }

    pub fn set_children(self, __value: Vec<DocChildInfoItem>, __repo: &mut InfoSetData) {
        set_children_of_parsed_doc_info_item(self, __value, __repo).unwrap()
    }

    pub fn notations(self, __repo: &InfoSetData) -> &Option<Vec<NotationInfoItem>> {
        notations_of_parsed_doc_info_item(self, __repo).unwrap()
    }

    pub fn notations_mut(self, __repo: &mut InfoSetData) -> &mut Option<Vec<NotationInfoItem>> {
        notations_of_parsed_doc_info_item_mut(self, __repo).unwrap()
    }

    pub fn set_notations(self, __value: Option<Vec<NotationInfoItem>>, __repo: &mut InfoSetData) {
        set_notations_of_parsed_doc_info_item(self, __value, __repo).unwrap()
    }

    pub fn unparsed_entities(self, __repo: &InfoSetData) -> &Vec<UEInfoItem> {
        unparsed_entities_of_parsed_doc_info_item(self, __repo).unwrap()
    }

    pub fn unparsed_entities_mut(self, __repo: &mut InfoSetData) -> &mut Vec<UEInfoItem> {
        unparsed_entities_of_parsed_doc_info_item_mut(self, __repo).unwrap()
    }

    pub fn set_unparsed_entities(self, __value: Vec<UEInfoItem>, __repo: &mut InfoSetData) {
        set_unparsed_entities_of_parsed_doc_info_item(self, __value, __repo).unwrap()
    }

    pub fn base_uri(self, __repo: &InfoSetData) -> Option<UriSpan> {
        base_uri_of_parsed_doc_info_item(self, __repo).unwrap()
    }

    pub fn set_base_uri(self, __value: Option<UriSpan>, __repo: &mut InfoSetData) {
        set_base_uri_of_parsed_doc_info_item(self, __value, __repo).unwrap()
    }

    pub fn all_declarations_processed(self, __repo: &InfoSetData) -> bool {
        all_declarations_processed_of_parsed_doc_info_item(self, __repo).unwrap()
    }

    pub fn set_all_declarations_processed(self, __value: bool, __repo: &mut InfoSetData) {
        set_all_declarations_processed_of_parsed_doc_info_item(self, __value, __repo).unwrap()
    }
}

#[repo::entity(repo = InfoSetData)]
pub struct ElementInfoItem {
    pub namespace_name: Option<Span>,
    pub local_name: Span,
    pub prefix: Option<Span>,
    #[by_ref]
    pub children: Vec<ElementChildInfoItem>,
    #[by_ref]
    pub attributes: Vec<AttributeInfoItem>,
    #[by_ref]
    pub namespace_attributes: Vec<AttributeInfoItem>,
    #[by_ref]
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
    #[by_ref]
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
