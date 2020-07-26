use super::*;
use xmlparser::Token as XmlToken;
use xmlparser::Tokenizer as XmlTokenizer;

pub enum ParseError {}

impl<'input> InfoSet<'input> {
    pub fn parse(input: &'input str) -> Result<Self, ParseError> {
        let repo = Repo::new();
        let xml_tokenizer = XmlTokenizer::from(input);
        let mut tokens = xml_tokenizer.into_iter();
        todo!()
    }
}