#![allow(unused_variables, unused_assignments, unused_mut)]

use super::*;
use core::iter::Peekable;
use thiserror::Error;
use xmlparser::StrSpan as XmlStrSpan;
use xmlparser::Token as XmlToken;
use xmlparser::Tokenizer as XmlTokenizer;

trait FromXmlStrSpan<'a> {
    fn from_xml_strspan(v: XmlStrSpan<'a>) -> Self;
}

impl<'a> FromXmlStrSpan<'_> for Span {
    fn from_xml_strspan(v: XmlStrSpan<'_>) -> Self {
        Span(v.range())
    }
}

impl FromXmlStrSpan<'_> for Option<Span> {
    fn from_xml_strspan(v: XmlStrSpan<'_>) -> Self {
        fn range_is_empty(v: &ops::Range<usize>) -> bool {
            !(v.start < v.end)
        }
        let r = v.range();
        if range_is_empty(&r) {
            None
        } else {
            Some(Span(r))
        }
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("token error: {0}")]
    TokenError(#[from] xmlparser::Error),
    #[error("unexpected token")]
    UnexpectedToken,
    #[error("unexpected eof")]
    UnexpectedEOF,
    #[error("duplicate ns attribute")]
    DuplicateNSAttribute,
    #[error("duplicate root element")]
    DuplicateRootElement,
}

pub fn parse<'input>(input: &'input str) -> Result<InfoSet<'input>, ParseError> {
    let (i, s) = parse_with_statistics(input)?;
    Ok(i)
}

pub fn parse_with_statistics<'input>(
    input: &'input str,
) -> Result<(InfoSet<'input>, InfoSetStatistics), ParseError> {
    let mut info_set_data = InfoSetData::default();
    let xml_tokenizer = XmlTokenizer::from(input);
    let mut tokens = xml_tokenizer.into_iter().peekable();
    let mut xmlinfoset_statistics = InfoSetStatistics::default();
    let doc_info_item = DocInfoItem::new_not_yet_parsed(&mut info_set_data);
    parse_xml_doc(&mut info_set_data, &mut tokens, doc_info_item)?;
    info_set_data.doc_info_item = Some(doc_info_item);
    let xmlinfoset = InfoSet {
        input: Cow::Borrowed(input),
        data: info_set_data,
    };
    Ok((xmlinfoset, xmlinfoset_statistics))
}

fn parse_xml_doc(
    repo: &mut InfoSetData,
    mut tokens: &mut Peekable<XmlTokenizer>,
    doc_info_item: DocInfoItem,
) -> Result<(), ParseError> {
    enum DocState {
        Initial,
        AfterXmlDecl,
        AfterDTD,
        AfterRootElement,
        Done,
    }
    let mut state;

    state = DocState::Initial;
    let (mut xml_version, mut xml_encoding, mut xml_standalone) = (None, None, None);

    if let Some(peeked_token) = tokens.peek() {
        let peeked_token = match peeked_token {
            Ok(peeked_token) => peeked_token,
            Err(e) => return Err(ParseError::TokenError(e.clone())),
        };
        if let XmlToken::Declaration { .. } = peeked_token {
            let token = tokens.next().unwrap().unwrap();
            if let XmlToken::Declaration {
                version,
                encoding,
                standalone,
                ..
            } = token
            {
                xml_version = Some(parse_version(version.as_str()));
                xml_encoding = encoding.map(|x| parse_encoding_scheme(x.as_str()));
                xml_standalone = standalone;
            }
        }
    };

    state = DocState::AfterXmlDecl;
    let mut root_element = None;
    while let Some(peeked_token) = tokens.peek() {
        let peeked_token = match peeked_token {
            Ok(peeked_token) => peeked_token,
            Err(e) => return Err(ParseError::TokenError(e.clone())),
        };
        match peeked_token {
            XmlToken::DtdStart { .. } | XmlToken::EmptyDtd { .. } => {
                if !matches!(state, DocState::AfterXmlDecl) {
                    return Err(ParseError::UnexpectedToken);
                }
                let dtd = parse_dtd(repo, &mut tokens)?;
                state = DocState::AfterDTD;
            }
            XmlToken::ElementStart { .. } => {
                if !matches!(state, DocState::AfterXmlDecl | DocState::AfterDTD) {
                    return Err(ParseError::UnexpectedToken);
                }
                let element = parse_element_tree(repo, &mut tokens, doc_info_item)?;
                root_element = Some(element);
                state = DocState::AfterRootElement;
            }
            XmlToken::Comment { .. } | XmlToken::ProcessingInstruction { .. } => {
                let misc_item = tokens.next().unwrap().unwrap();
                // FIXME
            }
            _ => {
                return Err(ParseError::UnexpectedToken);
            }
        };
    }

    if !matches!(state, DocState::AfterRootElement) {
        return Err(ParseError::UnexpectedEOF);
    }
    state = DocState::Done;

    doc_info_item.transition_to_parsed_from_not_yet_parsed(
        repo::keyed!(version: xml_version.unwrap_or(Version::Version1_0)),
        repo::keyed!(character_encoding_scheme: xml_encoding),
        repo::keyed!(standalone: xml_standalone),
        repo::keyed!(document_element: root_element.unwrap()),
        repo::keyed!(children: fixme_impl!(Vec::new())),
        repo::keyed!(notations: fixme_impl!(None)),
        repo::keyed!(unparsed_entities: fixme_impl!(Vec::new())),
        repo::keyed!(base_uri: fixme_impl!(None)),
        repo::keyed!(all_declarations_processed: fixme_impl!(true)),
        repo,
    );

    Ok(())
}

fn parse_dtd(
    repo: &mut InfoSetData,
    tokens: &mut Peekable<XmlTokenizer>,
) -> Result<(), ParseError> {
    todo!();
}

fn append_to_element_as_child(
    repo: &mut InfoSetData,
    parent: ElementInfoItem,
    v: impl Into<ElementChildInfoItem>,
) {
    let parent_children = parent.children_mut(repo);
    todo!();
}

fn parse_element_tree(
    repo: &mut InfoSetData,
    tokens: &mut Peekable<XmlTokenizer>,
    doc_info_item: DocInfoItem,
) -> Result<ElementInfoItem, ParseError> {
    struct ParseStackEntry<'a> {
        element_info_item: ElementInfoItem,
        prefix: XmlStrSpan<'a>,
        local_name: XmlStrSpan<'a>,
    }
    let mut parse_stack: Vec<ParseStackEntry<'_>> = Vec::new();
    let mut root = None;
    enum ParseState {
        Initial,
        AfterDescent,
        AfterAppend,
        AfterUnwind,
        Done,
    }
    let mut parse_state = ParseState::Initial;
    'parse_elem_tree: loop {
        let next_token = match tokens.next() {
            None => return Err(ParseError::UnexpectedEOF),
            Some(Err(e)) => return Err(ParseError::TokenError(e)),
            Some(Ok(t)) => t,
        };
        match next_token {
            XmlToken::ElementStart {
                prefix: element_prefix,
                local: element_local,
                ..
            } => {
                let self_close;
                let mut non_namespace_attrs = vec![];
                let mut namespace_attributes = vec![];
                let mut default_namespace_attribute = None;
                'parse_attr_list: loop {
                    use xmlparser::ElementEnd;
                    let iterate_item = tokens.next();
                    let iterate_token = match iterate_item {
                        None => return Err(ParseError::UnexpectedEOF),
                        Some(Err(e)) => return Err(ParseError::TokenError(e)),
                        Some(Ok(XmlToken::Attribute {
                            prefix: attr_prefix,
                            local: attr_local,
                            value: attr_value,
                            ..
                        })) => {
                            if attr_prefix.is_empty() && attr_local.as_str() == "xmlns" {
                                if default_namespace_attribute.is_some() {
                                    return Err(ParseError::DuplicateNSAttribute);
                                }
                                default_namespace_attribute = Some((attr_local, attr_value));
                            } else if attr_prefix.as_str() == "xmlns" {
                                let local_str = attr_local.as_str();
                                namespace_attributes.push((attr_local, attr_value));
                            } else {
                                non_namespace_attrs.push((attr_prefix, attr_local, attr_value));
                            }
                        }
                        Some(Ok(XmlToken::ElementEnd {
                            end: ElementEnd::Open,
                            ..
                        })) => {
                            self_close = false;
                            break 'parse_attr_list;
                        }
                        Some(Ok(XmlToken::ElementEnd {
                            end: ElementEnd::Empty,
                            ..
                        })) => {
                            self_close = true;
                            break 'parse_attr_list;
                        }
                        _ => return Err(ParseError::UnexpectedToken),
                    };
                }
                let parent = if let Some(e) = parse_stack.last() {
                    Some(e.element_info_item)
                } else {
                    None
                };
                let element_info_item = ElementInfoItem::new(
                    fixme_impl!(None),
                    Span::from_xml_strspan(element_local),
                    Option::<Span>::from_xml_strspan(element_prefix),
                    fixme_impl!(Vec::new()),
                    fixme_impl!(Vec::new()),
                    fixme_impl!(Vec::new()),
                    fixme_impl!(Vec::new()),
                    fixme_impl!(None),
                    if let Some(e) = parent {
                        ElementParentInfoItem::Element(e)
                    } else {
                        ElementParentInfoItem::Doc(doc_info_item)
                    },
                    repo,
                );
                if let Some(parent) = parent {
                    append_to_element_as_child(repo, parent, element_info_item);
                } else {
                    if root.is_some() {
                        return Err(ParseError::DuplicateRootElement);
                    }
                    root = Some(element_info_item);
                }
                if !self_close {
                    parse_stack.push(ParseStackEntry {
                        element_info_item,
                        prefix: element_prefix,
                        local_name: element_local,
                    });
                    parse_state = ParseState::AfterDescent;
                } else {
                    if parent.is_some() {
                        parse_state = ParseState::AfterAppend;
                    } else {
                        parse_state = ParseState::Done;
                        break 'parse_elem_tree;
                    }
                }
            }
            XmlToken::Comment { text, .. } => {
                if !matches!(
                    parse_state,
                    ParseState::AfterDescent | ParseState::AfterAppend | ParseState::AfterUnwind
                ) {
                    return Err(ParseError::UnexpectedToken);
                }
                let parent = parse_stack.last().unwrap().element_info_item;
                let comment_info_item = CommentInfoItem::new(
                    Span::from_xml_strspan(text),
                    CommentParentInfoItem::Element(parent),
                    repo,
                );
                append_to_element_as_child(repo, parent, comment_info_item);
                parse_state = ParseState::AfterAppend;
            }
            XmlToken::ProcessingInstruction {
                target, content, ..
            } => {
                if !matches!(
                    parse_state,
                    ParseState::AfterDescent | ParseState::AfterAppend | ParseState::AfterUnwind
                ) {
                    return Err(ParseError::UnexpectedToken);
                }

                let parent = parse_stack.last().unwrap().element_info_item;
                let pi_info_item = PIInfoItem::new(
                    Span::from_xml_strspan(target),
                    content.map(Span::from_xml_strspan),
                    fixme_impl!(None),
                    fixme_impl!(None),
                    PIParentInfoItem::Element(parent),
                    repo,
                );
                append_to_element_as_child(repo, parent, pi_info_item);
                parse_state = ParseState::AfterAppend;
            }
            XmlToken::ElementEnd { end, .. } => {
                use xmlparser::ElementEnd;
                if !matches!(
                    parse_state,
                    ParseState::AfterDescent | ParseState::AfterAppend | ParseState::AfterUnwind
                ) {
                    return Err(ParseError::UnexpectedToken);
                }
                let (prefix, local_name) = match end {
                    ElementEnd::Close(prefix, local_name) => (prefix, local_name),
                    _ => return Err(ParseError::UnexpectedToken),
                };
                match parse_stack.pop() {
                    Some(entry) if entry.prefix == prefix && entry.local_name == local_name => {
                        // do nothing.
                    }
                    _ => return Err(ParseError::UnexpectedToken),
                }
                if parse_stack.is_empty() {
                    parse_state = ParseState::Done;
                    break 'parse_elem_tree;
                }
                parse_state = ParseState::AfterUnwind;
            }
            _ => return Err(ParseError::UnexpectedToken),
        }
    }
    debug_assert!(matches!(parse_state, ParseState::Done));
    Ok(root.unwrap())
}

fn parse_version(version: &str) -> Version {
    if version == "1.0" {
        Version::Version1_0
    } else if version == "1.1" {
        Version::Version1_1
    } else {
        Version::Other(version.to_owned())
    }
}

fn parse_encoding_scheme(encoding_scheme: &str) -> EncodingScheme {
    if encoding_scheme == "UTF-8" {
        EncodingScheme::Utf8
    } else {
        EncodingScheme::Other(encoding_scheme.to_owned())
    }
}
