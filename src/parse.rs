use super::*;
use core::iter::Peekable;
use repository::Error as RepoError;
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
    #[error("internal error")]
    InternalError(#[from] RepoError),
}

impl<T: core::any::Any, E> From<repository::prealloc_tx::PreallocTxError<T, E>> for ParseError {
    fn from(v: repository::prealloc_tx::PreallocTxError<T, E>) -> Self {
        ParseError::InternalError(RepoError::InvalidPtr)
    }
}

pub fn parse<'input>(input: &'input str) -> Result<InfoSet<'input>, ParseError> {
    let (i, s) = parse_with_statistics(input)?;
    Ok(i)
}

pub fn parse_with_statistics<'input>(
    input: &'input str,
) -> Result<(InfoSet<'input>, InfoSetStatistics), ParseError> {
    let mut repo = Repo::new();
    let xml_tokenizer = XmlTokenizer::from(input);
    let mut tokens = xml_tokenizer.into_iter().peekable();
    let mut xmlinfoset_statistics = InfoSetStatistics::default();
    let doc_info_item_ptr =
        repo.transaction_preallocate(|tx| -> Result<DocInfoItemPtr, ParseError> {
            let doc_info_item_ptr = tx.preallocate::<DocInfoItem>().cast_repo::<InfoSetData>();
            let doc_info_item = parse_xml_doc(tx.repo_mut(), &mut tokens, doc_info_item_ptr)?;
            tx.init_preallocation(doc_info_item_ptr, doc_info_item)?;
            Ok(doc_info_item_ptr)
        })?;
    let xmlinfoset = InfoSet {
        input: Cow::Borrowed(input),
        data: InfoSetData {
            repo,
            doc_info_item: doc_info_item_ptr,
        },
    };
    Ok((xmlinfoset, xmlinfoset_statistics))
}

fn parse_xml_doc(
    repo: &mut Repo,
    mut tokens: &mut Peekable<XmlTokenizer>,
    doc_info_item_ptr: DocInfoItemPtr,
) -> Result<DocInfoItem, ParseError> {
    enum DocState {
        Initial,
        AfterXmlDecl,
        AfterDTD,
        AfterRootElement,
        Done,
    }

    let mut state = DocState::Initial;

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
                let element = parse_element_tree(repo, &mut tokens, doc_info_item_ptr)?;
                root_element = Some(element);
                state = DocState::AfterRootElement;
            }
            XmlToken::Comment { .. } | XmlToken::ProcessingInstruction { .. } => {
                let misc_item = tokens.next().unwrap().unwrap();
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
    let doc_info_item = DocInfoItem {
        version: xml_version.unwrap_or(Version::Version1_0),
        character_encoding_scheme: xml_encoding,
        standalone: xml_standalone,
        document_element: root_element.unwrap(),
        children: todo!(),
        notations: todo!(),
        unparsed_entities: todo!(),
        base_uri: todo!(),
        all_declarations_processed: todo!(),
    };
    Ok(doc_info_item)
}

fn parse_dtd(repo: &mut Repo, tokens: &mut Peekable<XmlTokenizer>) -> Result<(), ParseError> {
    todo!();
}

fn append_to_element_as_child(
    repo: &mut Repo,
    parent: ElementInfoItemPtr,
    v: impl Into<ElementChildInfoItemPtr>,
) -> Result<(), RepoError> {
    let parent_mut = parent.cast_repo::<Repo>().get_mut(repo)?;
    todo!();
    Ok(())
}

fn parse_element_tree(
    repo: &mut Repo,
    tokens: &mut Peekable<XmlTokenizer>,
    doc_info_item_ptr: DocInfoItemPtr,
) -> Result<ElementInfoItemPtr, ParseError> {
    struct ParseStackEntry<'a> {
        element_ptr: ElementInfoItemPtr,
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
            XmlToken::ElementStart { prefix, local, .. } => {
                let self_close;
                'parse_attr_list: loop {
                    use xmlparser::ElementEnd;
                    let iterate_item = tokens.next();
                    let iterate_token = match iterate_item {
                        None => return Err(ParseError::UnexpectedEOF),
                        Some(Err(e)) => return Err(ParseError::TokenError(e)),
                        Some(Ok(XmlToken::Attribute {
                            prefix: attr_prefix,
                            local: attr_local,
                            ..
                        })) => {
                            todo!();
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
                    Some(e.element_ptr)
                } else {
                    None
                };
                let element_info_item = ElementInfoItem {
                    prefix: Option::<Span>::from_xml_strspan(prefix),
                    local_name: Span::from_xml_strspan(local),
                    base_uri: todo!(),
                    parent: if let Some(e) = parent {
                        ElementParentInfoItemPtr::Element(e)
                    } else {
                        ElementParentInfoItemPtr::Doc(doc_info_item_ptr)
                    },
                    namespace_name: todo!(),
                    children: todo!(),
                    attributes: todo!(),
                    namespace_attributes: todo!(),
                    in_scope_namespaces: todo!(),
                };
                todo!();
            }
            XmlToken::Comment { text, .. } => {
                if !matches!(
                    parse_state,
                    ParseState::AfterDescent | ParseState::AfterAppend | ParseState::AfterUnwind
                ) {
                    return Err(ParseError::UnexpectedToken);
                }
                let parent = parse_stack.last().unwrap().element_ptr;
                let comment_info_item = CommentInfoItem {
                    content: Span::from_xml_strspan(text),
                    parent: CommentParentInfoItemPtr::Element(parent),
                };
                let comment_info_ptr = repo.insert(comment_info_item).cast_repo::<InfoSetData>();
                append_to_element_as_child(repo, parent, comment_info_ptr)?;
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

                let parent = parse_stack.last().unwrap().element_ptr;
                let pi_info_item = PIInfoItem {
                    target: Span::from_xml_strspan(target),
                    content: content.map(Span::from_xml_strspan),
                    base_uri: todo!(),
                    notation: todo!(),
                    parent: PIParentInfoItemPtr::Element(parent),
                };
                let pi_info_ptr = repo.insert(pi_info_item).cast_repo::<InfoSetData>();
                append_to_element_as_child(repo, parent, pi_info_ptr)?;
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
