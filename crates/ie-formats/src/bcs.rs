use ie_core::{IdsResolver, ResourceType};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct BcsJson {
    pub resource_type: String,
    pub resource_name: String,
    pub blocks: Vec<BcsConditionResponseJson>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BcsConditionResponseJson {
    pub triggers: Vec<BcsTriggerJson>,
    pub responses: Vec<BcsResponseJson>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BcsTriggerJson {
    pub opcode: i32,
    pub name: Option<String>,
    pub negated: bool,
    pub int_args: [i32; 4],
    pub string_args: [String; 2],
    pub object: BcsObjectJson,
}

#[derive(Debug, Clone, Serialize)]
pub struct BcsActionJson {
    pub leading: i32,
    pub opcode: i32,
    pub name: Option<String>,
    pub int_args: [i32; 4],
    pub string_args: [String; 2],
    pub objects: [BcsObjectJson; 3],
    pub point: Option<BcsPointJson>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BcsPointJson {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct BcsResponseJson {
    pub weight: i32,
    pub actions: Vec<BcsActionJson>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BcsObjectJson {
    pub raw: [i32; 12],
    pub name: Option<String>,
    pub decoded: BcsObjectDecodedJson,
}

#[derive(Debug, Clone, Serialize)]
pub struct BcsObjectDecodedJson {
    pub ea: Option<String>,
    pub general: Option<String>,
    pub race: Option<String>,
    pub class: Option<String>,
    pub specific: Option<String>,
    pub gender: Option<String>,
    pub alignment: Option<String>,
    pub identifier: Option<String>,
    pub extra_targets: Option<[i32; 4]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BcsSourcePos {
    pub byte_offset: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Error)]
pub enum BcsParseError {
    #[error(
        "invalid BCS token at line {line}, column {column}: {message}",
        line = .pos.line,
        column = .pos.column
    )]
    InvalidToken { pos: BcsSourcePos, message: String },
    #[error(
        "unexpected token at line {line}, column {column}: {message}",
        line = .pos.line,
        column = .pos.column
    )]
    UnexpectedToken { pos: BcsSourcePos, message: String },
    #[error("unexpected end of BCS input: {message}")]
    UnexpectedEof { message: String },
}

impl From<BcsParseError> for crate::FormatError {
    fn from(err: BcsParseError) -> Self {
        crate::FormatError::Parse(err.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tag {
    Sc,
    Cr,
    Co,
    Tr,
    Rs,
    Re,
    Ac,
    Ob,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    Tag(Tag),
    Int(i32),
    Str(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    pos: BcsSourcePos,
}

pub fn parse_bcs(
    bytes: &[u8],
    resource_name: &str,
    resolver: Option<&dyn IdsResolver>,
) -> Result<BcsJson, crate::FormatError> {
    let tokens = tokenize(bytes)?;
    let mut parser = ParserState::new(&tokens, resolver);
    let blocks = parser.parse_sc()?;
    parser.expect_end()?;

    Ok(BcsJson {
        resource_type: ResourceType::Bcs.as_str().to_string(),
        resource_name: resource_name.to_string(),
        blocks,
    })
}

struct ParserState<'a> {
    tokens: &'a [Token],
    index: usize,
    resolver: Option<&'a dyn IdsResolver>,
}

impl<'a> ParserState<'a> {
    fn new(tokens: &'a [Token], resolver: Option<&'a dyn IdsResolver>) -> Self {
        Self {
            tokens,
            index: 0,
            resolver,
        }
    }

    fn parse_sc(&mut self) -> Result<Vec<BcsConditionResponseJson>, BcsParseError> {
        self.expect_tag(Tag::Sc)?;

        let mut blocks = Vec::new();
        while !self.next_is_tag(Tag::Sc) {
            blocks.push(self.parse_cr()?);
        }

        self.expect_tag(Tag::Sc)?;
        Ok(blocks)
    }

    fn parse_cr(&mut self) -> Result<BcsConditionResponseJson, BcsParseError> {
        self.expect_tag(Tag::Cr)?;
        let triggers = self.parse_co()?;
        let responses = self.parse_rs()?;
        self.expect_tag(Tag::Cr)?;

        Ok(BcsConditionResponseJson {
            triggers,
            responses,
        })
    }

    fn parse_co(&mut self) -> Result<Vec<BcsTriggerJson>, BcsParseError> {
        self.expect_tag(Tag::Co)?;

        let mut triggers = Vec::new();
        while !self.next_is_tag(Tag::Co) {
            triggers.push(self.parse_tr()?);
        }

        self.expect_tag(Tag::Co)?;
        Ok(triggers)
    }

    fn parse_tr(&mut self) -> Result<BcsTriggerJson, BcsParseError> {
        self.expect_tag(Tag::Tr)?;

        let raw_opcode = self.expect_int()?;
        let int_args = [
            self.expect_int()?,
            self.expect_int()?,
            self.expect_int()?,
            self.expect_int()?,
        ];
        let string_args = [self.expect_string()?, self.expect_string()?];
        self.expect_tag(Tag::Ob)?;
        let object = self.parse_object()?;
        self.expect_tag(Tag::Ob)?;
        self.expect_tag(Tag::Tr)?;

        let negated = raw_opcode < 0;
        let lookup_opcode = raw_opcode.checked_abs().unwrap_or(raw_opcode);

        Ok(BcsTriggerJson {
            opcode: raw_opcode,
            name: self
                .resolver
                .and_then(|resolver| resolver.resolve_trigger(lookup_opcode)),
            negated,
            int_args,
            string_args,
            object,
        })
    }

    fn parse_rs(&mut self) -> Result<Vec<BcsResponseJson>, BcsParseError> {
        self.expect_tag(Tag::Rs)?;

        let mut responses = Vec::new();
        while !self.next_is_tag(Tag::Rs) {
            responses.push(self.parse_re()?);
        }

        self.expect_tag(Tag::Rs)?;
        Ok(responses)
    }

    fn parse_re(&mut self) -> Result<BcsResponseJson, BcsParseError> {
        self.expect_tag(Tag::Re)?;
        let weight = self.expect_int()?;

        let mut actions = Vec::new();
        while !self.next_is_tag(Tag::Re) {
            actions.push(self.parse_ac()?);
        }

        self.expect_tag(Tag::Re)?;
        Ok(BcsResponseJson { weight, actions })
    }

    fn parse_ac(&mut self) -> Result<BcsActionJson, BcsParseError> {
        self.expect_tag(Tag::Ac)?;

        let leading = self.expect_int()?;

        self.expect_tag(Tag::Ob)?;
        let object_1 = self.parse_object()?;
        self.expect_tag(Tag::Ob)?;

        self.expect_tag(Tag::Ob)?;
        let object_2 = self.parse_object()?;
        self.expect_tag(Tag::Ob)?;

        self.expect_tag(Tag::Ob)?;
        let object_3 = self.parse_object()?;
        self.expect_tag(Tag::Ob)?;

        let opcode = self.expect_int()?;
        let int_args = [
            self.expect_int()?,
            self.expect_int()?,
            self.expect_int()?,
            self.expect_int()?,
        ];
        let string_args = [self.expect_string()?, self.expect_string()?];
        let point = if self.next_is_int() {
            Some(BcsPointJson {
                x: self.expect_int()?,
                y: self.expect_int()?,
            })
        } else {
            None
        };
        self.expect_tag(Tag::Ac)?;

        Ok(BcsActionJson {
            leading,
            opcode,
            name: self
                .resolver
                .and_then(|resolver| resolver.resolve_action(leading)),
            int_args,
            string_args,
            objects: [object_1, object_2, object_3],
            point,
        })
    }

    fn parse_object(&mut self) -> Result<BcsObjectJson, BcsParseError> {
        let raw = [
            self.expect_int()?,
            self.expect_int()?,
            self.expect_int()?,
            self.expect_int()?,
            self.expect_int()?,
            self.expect_int()?,
            self.expect_int()?,
            self.expect_int()?,
            self.expect_int()?,
            self.expect_int()?,
            self.expect_int()?,
            self.expect_int()?,
        ];
        let name = self.expect_string()?;

        Ok(BcsObjectJson {
            raw,
            name: if name.is_empty() { None } else { Some(name) },
            decoded: decode_object(raw, self.resolver),
        })
    }

    fn expect_tag(&mut self, expected: Tag) -> Result<(), BcsParseError> {
        let token = self.next_token().ok_or_else(|| BcsParseError::UnexpectedEof {
            message: format!("expected tag {}", expected.as_str()),
        })?;

        match token.kind {
            TokenKind::Tag(actual) if actual == expected => Ok(()),
            _ => Err(BcsParseError::UnexpectedToken {
                pos: token.pos,
                message: format!(
                    "expected tag {}, found {}",
                    expected.as_str(),
                    describe_token_kind(&token.kind)
                ),
            }),
        }
    }

    fn expect_int(&mut self) -> Result<i32, BcsParseError> {
        let token = self.next_token().ok_or_else(|| BcsParseError::UnexpectedEof {
            message: "expected integer".to_string(),
        })?;

        match &token.kind {
            TokenKind::Int(value) => Ok(*value),
            _ => Err(BcsParseError::UnexpectedToken {
                pos: token.pos,
                message: format!("expected integer, found {}", describe_token_kind(&token.kind)),
            }),
        }
    }

    fn expect_string(&mut self) -> Result<String, BcsParseError> {
        let token = self.next_token().ok_or_else(|| BcsParseError::UnexpectedEof {
            message: "expected quoted string".to_string(),
        })?;

        match &token.kind {
            TokenKind::Str(value) => Ok(value.clone()),
            _ => Err(BcsParseError::UnexpectedToken {
                pos: token.pos,
                message: format!(
                    "expected quoted string, found {}",
                    describe_token_kind(&token.kind)
                ),
            }),
        }
    }

    fn next_is_tag(&self, expected: Tag) -> bool {
        matches!(
            self.tokens.get(self.index).map(|token| &token.kind),
            Some(TokenKind::Tag(actual)) if *actual == expected
        )
    }

    fn next_is_int(&self) -> bool {
        matches!(
            self.tokens.get(self.index).map(|token| &token.kind),
            Some(TokenKind::Int(_))
        )
    }

    fn next_token(&mut self) -> Option<&'a Token> {
        let token = self.tokens.get(self.index)?;
        self.index += 1;
        Some(token)
    }

    fn expect_end(&self) -> Result<(), BcsParseError> {
        if let Some(token) = self.tokens.get(self.index) {
            Err(BcsParseError::UnexpectedToken {
                pos: token.pos,
                message: format!(
                    "unexpected trailing {}",
                    describe_token_kind(&token.kind)
                ),
            })
        } else {
            Ok(())
        }
    }
}

fn tokenize(bytes: &[u8]) -> Result<Vec<Token>, BcsParseError> {
    let mut tokens = Vec::new();
    let mut index = 0usize;
    let mut line = 1usize;
    let mut column = 1usize;

    while index < bytes.len() {
        let byte = bytes[index];

        match byte {
            b' ' | b'\t' => {
                index += 1;
                column += 1;
            }
            b'\r' => {
                index += 1;
                if index < bytes.len() && bytes[index] == b'\n' {
                    index += 1;
                }
                line += 1;
                column = 1;
            }
            b'\n' => {
                index += 1;
                line += 1;
                column = 1;
            }
            b'"' => {
                let start = BcsSourcePos {
                    byte_offset: index,
                    line,
                    column,
                };
                index += 1;
                column += 1;
                let string_start = index;

                while index < bytes.len() && bytes[index] != b'"' {
                    match bytes[index] {
                        b'\r' => {
                            return Err(BcsParseError::InvalidToken {
                                pos: start,
                                message: "unterminated quoted string".to_string(),
                            });
                        }
                        b'\n' => {
                            return Err(BcsParseError::InvalidToken {
                                pos: start,
                                message: "unterminated quoted string".to_string(),
                            });
                        }
                        _ => {
                            index += 1;
                            column += 1;
                        }
                    }
                }

                if index >= bytes.len() {
                    return Err(BcsParseError::UnexpectedEof {
                        message: format!(
                            "unterminated quoted string starting at line {}, column {}",
                            start.line, start.column
                        ),
                    });
                }

                let value = String::from_utf8_lossy(&bytes[string_start..index]).to_string();
                index += 1;
                column += 1;
                tokens.push(Token {
                    kind: TokenKind::Str(value),
                    pos: start,
                });
            }
            b'-' | b'0'..=b'9' => {
                let start = BcsSourcePos {
                    byte_offset: index,
                    line,
                    column,
                };
                let mut end = index;

                if bytes[end] == b'-' {
                    end += 1;
                    if end >= bytes.len() || !bytes[end].is_ascii_digit() {
                        return Err(BcsParseError::InvalidToken {
                            pos: start,
                            message: "expected digits after '-'".to_string(),
                        });
                    }
                }

                while end < bytes.len() && bytes[end].is_ascii_digit() {
                    end += 1;
                }

                let raw = std::str::from_utf8(&bytes[index..end]).map_err(|err| {
                    BcsParseError::InvalidToken {
                        pos: start,
                        message: format!("invalid integer token: {err}"),
                    }
                })?;
                let value = raw.parse::<i32>().map_err(|err| BcsParseError::InvalidToken {
                    pos: start,
                    message: format!("invalid integer '{raw}': {err}"),
                })?;

                tokens.push(Token {
                    kind: TokenKind::Int(value),
                    pos: start,
                });
                column += end - index;
                index = end;
            }
            b'A'..=b'Z' => {
                let start = BcsSourcePos {
                    byte_offset: index,
                    line,
                    column,
                };

                if let Some(tag) = match_tag(bytes, index) {
                    tokens.push(Token {
                        kind: TokenKind::Tag(tag),
                        pos: start,
                    });
                    index += 2;
                    column += 2;
                    continue;
                }

                let mut end = index + 1;
                while end < bytes.len() && bytes[end].is_ascii_alphabetic() {
                    end += 1;
                }
                let raw = String::from_utf8_lossy(&bytes[index..end]).to_string();
                return Err(BcsParseError::InvalidToken {
                    pos: start,
                    message: format!("unknown token '{raw}'"),
                });
            }
            _ => {
                let pos = BcsSourcePos {
                    byte_offset: index,
                    line,
                    column,
                };
                return Err(BcsParseError::InvalidToken {
                    pos,
                    message: format!("unexpected byte 0x{byte:02X}"),
                });
            }
        }
    }

    Ok(tokens)
}

fn match_tag(bytes: &[u8], index: usize) -> Option<Tag> {
    let slice = bytes.get(index..index + 2)?;
    match slice {
        b"SC" => Some(Tag::Sc),
        b"CR" => Some(Tag::Cr),
        b"CO" => Some(Tag::Co),
        b"TR" => Some(Tag::Tr),
        b"RS" => Some(Tag::Rs),
        b"RE" => Some(Tag::Re),
        b"AC" => Some(Tag::Ac),
        b"OB" => Some(Tag::Ob),
        _ => None,
    }
}

fn decode_object(raw: [i32; 12], resolver: Option<&dyn IdsResolver>) -> BcsObjectDecodedJson {
    let resolve = |file: &str, value: i32| {
        if value == 0 {
            None
        } else {
            resolver.and_then(|resolver| resolver.resolve_ids(file, value))
        }
    };

    let extra_targets = [raw[8], raw[9], raw[10], raw[11]];

    BcsObjectDecodedJson {
        ea: resolve("EA", raw[0]),
        general: resolve("GENERAL", raw[1]),
        race: resolve("RACE", raw[2]),
        class: resolve("CLASS", raw[3]),
        specific: resolve("SPECIFIC", raw[4]),
        gender: resolve("GENDER", raw[5]),
        alignment: resolve("ALIGN", raw[6]),
        identifier: resolve("OBJECT", raw[7]),
        extra_targets: extra_targets.iter().any(|value| *value != 0).then_some(extra_targets),
    }
}

fn describe_token_kind(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Tag(tag) => format!("tag {}", tag.as_str()),
        TokenKind::Int(value) => format!("integer {value}"),
        TokenKind::Str(value) => format!("string {:?}", value),
    }
}

impl Tag {
    fn as_str(self) -> &'static str {
        match self {
            Tag::Sc => "SC",
            Tag::Cr => "CR",
            Tag::Co => "CO",
            Tag::Tr => "TR",
            Tag::Rs => "RS",
            Tag::Re => "RE",
            Tag::Ac => "AC",
            Tag::Ob => "OB",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NullResolver;

    impl IdsResolver for NullResolver {
        fn resolve_trigger(&self, _opcode: i32) -> Option<String> {
            None
        }

        fn resolve_action(&self, _opcode: i32) -> Option<String> {
            None
        }

        fn resolve_ids(&self, _file: &str, _value: i32) -> Option<String> {
            None
        }
    }

    struct TestResolver;

    impl IdsResolver for TestResolver {
        fn resolve_trigger(&self, opcode: i32) -> Option<String> {
            match opcode {
                1 => Some("Global".to_string()),
                2 => Some("NumTimesTalkedTo".to_string()),
                _ => None,
            }
        }

        fn resolve_action(&self, opcode: i32) -> Option<String> {
            match opcode {
                30 => Some("SetGlobal".to_string()),
                40 => Some("DisplayStringHead".to_string()),
                60 => Some("GiveItem".to_string()),
                _ => None,
            }
        }

        fn resolve_ids(&self, file: &str, value: i32) -> Option<String> {
            match (file, value) {
                ("EA", 2) => Some("PC".to_string()),
                ("OBJECT", -1) => Some("LastAttackerOf".to_string()),
                ("OBJECT", 7) => Some("Myself".to_string()),
                _ => None,
            }
        }
    }

    #[test]
    fn parses_synthetic_bcs_with_nested_blocks_weights_and_names() {
        let bytes = br#"SC
CR
CO
TR
1 10 20 30 40 "PLOT" "GLOBAL" OB
2 0 0 0 0 0 0 7 0 0 0 0 "Myself" OB
TR
TR
-2 1 2 3 4 "" ""OB
0 0 0 0 0 0 0 -1 0 0 0 0 ""OB
TR
CO
RS
RE
100AC
30OB
2 0 0 0 0 0 0 0 0 0 0 0 ""OB
OB
0 0 0 0 0 0 0 0 0 0 0 0 ""OB
OB
0 0 0 0 0 0 0 0 1 2 3 4 "DV"OB
5 7 8 9 10 "RASAAD_PLOT" "GLOBAL" AC
AC
40OB
0 0 0 0 0 0 0 0 0 0 0 0 ""OB
OB
0 0 0 0 0 0 0 0 0 0 0 0 ""OB
OB
0 0 0 0 0 0 0 0 0 0 0 0 ""OB
6 0 0 0 0 "" "" 0 0 AC
RE
RE
50AC
60OB
0 0 0 0 0 0 0 0 0 0 0 0 ""OB
OB
0 0 0 0 0 0 0 0 0 0 0 0 ""OB
OB
0 0 0 0 0 0 0 0 0 0 0 0 ""OB
6 0 0 0 0 "" "" AC
RE
RS
CR
SC"#;

        let bcs = parse_bcs(bytes, "RASAAD.BCS", Some(&TestResolver)).expect("BCS should parse");

        assert_eq!(bcs.resource_name, "RASAAD.BCS");
        assert_eq!(bcs.resource_type, "BCS");
        assert_eq!(bcs.blocks.len(), 1);

        let block = &bcs.blocks[0];
        assert_eq!(block.triggers.len(), 2);
        assert_eq!(block.responses.len(), 2);

        let trigger_0 = &block.triggers[0];
        assert_eq!(trigger_0.name.as_deref(), Some("Global"));
        assert!(!trigger_0.negated);
        assert_eq!(trigger_0.int_args, [10, 20, 30, 40]);
        assert_eq!(trigger_0.string_args, ["PLOT".to_string(), "GLOBAL".to_string()]);
        assert_eq!(trigger_0.object.decoded.ea.as_deref(), Some("PC"));
        assert_eq!(
            trigger_0.object.decoded.identifier.as_deref(),
            Some("Myself")
        );
        assert_eq!(trigger_0.object.name.as_deref(), Some("Myself"));

        let trigger_1 = &block.triggers[1];
        assert_eq!(trigger_1.name.as_deref(), Some("NumTimesTalkedTo"));
        assert!(trigger_1.negated);
        assert_eq!(
            trigger_1.object.decoded.identifier.as_deref(),
            Some("LastAttackerOf")
        );

        let response_0 = &block.responses[0];
        assert_eq!(response_0.weight, 100);
        assert_eq!(response_0.actions.len(), 2);
        assert_eq!(response_0.actions[0].leading, 30);
        assert_eq!(response_0.actions[0].name.as_deref(), Some("SetGlobal"));
        assert_eq!(response_0.actions[0].objects[2].name.as_deref(), Some("DV"));
        assert_eq!(
            response_0.actions[0].objects[2].decoded.extra_targets,
            Some([1, 2, 3, 4])
        );
        assert!(response_0.actions[0].point.is_none());
        assert_eq!(response_0.actions[1].leading, 40);
        assert_eq!(
            response_0.actions[1].name.as_deref(),
            Some("DisplayStringHead")
        );
        assert_eq!(response_0.actions[1].point.as_ref().map(|p| p.x), Some(0));
        assert_eq!(response_0.actions[1].point.as_ref().map(|p| p.y), Some(0));

        let response_1 = &block.responses[1];
        assert_eq!(response_1.weight, 50);
        assert_eq!(response_1.actions[0].leading, 60);
        assert_eq!(
            response_1.actions[0].name.as_deref(),
            Some("GiveItem")
        );
        assert!(response_1.actions[0].point.is_none());
    }

    #[test]
    fn parses_without_resolver_and_leaves_names_empty() {
        let bytes = br#"SC CR CO TR 1 0 0 0 0 "" "" OB 0 0 0 0 0 0 0 0 0 0 0 0 "" OB TR CO RS RE 1AC 30OB 0 0 0 0 0 0 0 0 0 0 0 0 "" OBOB 0 0 0 0 0 0 0 0 0 0 0 0 "" OBOB 0 0 0 0 0 0 0 0 0 0 0 0 "" OB 5 0 0 0 0 "" "" AC RE RS CR SC"#;

        let bcs = parse_bcs(bytes, "NORES.BCS", Some(&NullResolver)).expect("BCS should parse");
        let block = &bcs.blocks[0];
        assert!(block.triggers[0].name.is_none());
        assert!(block.responses[0].actions[0].name.is_none());
        assert!(block.triggers[0].object.decoded.identifier.is_none());
        assert_eq!(block.responses[0].actions[0].leading, 30);
        assert!(block.responses[0].actions[0].point.is_none());
    }

    #[test]
    fn reports_malformed_truncated_trigger_without_panicking() {
        let bytes = br#"SC CR CO TR 1 0 0 0 0 "" "" OB 0 0 0 0 0 0 0 0 0 0 0 0 "" OB CO RS RE 1 RE RS CR SC"#;

        let err = parse_bcs(bytes, "BAD.BCS", None).expect_err("BCS parse should fail");
        let message = format!("{err}");
        assert!(message.contains("expected tag TR"), "unexpected error: {message}");
    }
}
