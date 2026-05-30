use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::assets::{AssetDb, AssetKind};
use crate::game_config::{Config, ConfigId};
use crate::game_config::effects::EffectMechanicConfig;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub enum ParameterType {
    #[default]
    Constant,
    Expression(String)
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ParameterOperator {
    Plus,
    Minus,
    Mul,
    Div,
    Clamp,
    Min,
    Max,
    Round,
    Rand
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ExpressionParameterNode {
    /// `{имя_черты}`
    ParameterValue(ConfigId<ParameterConfig>),
    /// `[имя_лычки]`
    TagCount(ConfigId<TagConfig>),
    /// `123.456789`
    Constant(f32),
    /// `(+ 123.456789 {x} [y] (* 2.0 6.0))`
    Operator(ParameterOperator, Vec<ExpressionParameterNode>)
}
impl Default for ExpressionParameterNode {
    fn default() -> Self { ExpressionParameterNode::Constant(0.0) }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub enum CompiledExpressionParameterNode {
    #[default]
    None,
    Error { compile_error: String },
    Ok(ExpressionParameterNode)
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct ParameterConfig {
    /// Имя для формул
    pub bound_name: String,
    /// Название в игре
    pub name: String,
    /// Описание в игре
    pub description: String,
    /// Иконка для информационных окон
    pub sprite_name : String,
    /// Тип черты
    pub parameter_type: ParameterType,
    /// В случае если черта вычисляемая, содержит скомпилированное выражение
    /// (или ошибку, если не удалось скомпилировать)
    compiled_expression: CompiledExpressionParameterNode
}

impl Config for ParameterConfig {}

use crate::app::editor_stage::image_widgets::SpriteHolder;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct TagConfig {
    pub bound_name: String,
    pub name: String,
    pub description: String,
    pub sprite_name : String,
    #[serde(default)]
    pub sprite_pivot: [u8; 2],
    pub effect_mechanic: Option<ConfigId<EffectMechanicConfig>>,
}

impl SpriteHolder for TagConfig {
    fn sprite_name(&self) -> &str {
        &self.sprite_name
    }
    fn sprite_pivot(&self) -> &[u8; 2] {
        &self.sprite_pivot
    }
    fn sprite_pivot_mut(&mut self) -> &mut [u8; 2] {
        &mut self.sprite_pivot
    }
}

impl Config for TagConfig {}

pub struct ExpressionParameterIdCache {
    tags: HashMap<String, ConfigId<TagConfig>>,
    parameters: HashMap<String, ConfigId<ParameterConfig>>,
    is_test: bool,
}
impl ExpressionParameterIdCache {
    pub fn new() -> Self {
        Self {
            tags: HashMap::new(),
            parameters: HashMap::new(),
            is_test: false,
        }
    }

    fn resolve_tag_id(&mut self, asset_db: &AssetDb, tag_name: &str) {
        if self.is_test {
            // в тестах ассетов нет и разрешать нечего
            return;
        }
        for (uuid, _) in asset_db.list_all_assets(AssetKind::TagConfig) {
            let asset_text = asset_db.load_json5_asset(AssetKind::TagConfig, uuid);
            let tag_config = json5::from_str::<TagConfig>(&asset_text).ok()
                .expect("Failed to parse tag config");
            if tag_name == tag_config.bound_name {
                self.tags.insert(tag_name.to_string(), ConfigId::from_uuid(uuid));
                break;
            }
        }
    }

    fn resolve_parameter_id(&mut self, asset_db: &AssetDb, parameter_name: &str) {
        if self.is_test {
            // в тестах ассетов нет и разрешать нечего
            return;
        }
        for (uuid, _) in asset_db.list_all_assets(AssetKind::ParameterConfig) {
            let asset_text = asset_db.load_json5_asset(AssetKind::ParameterConfig, uuid);
            let parameter_config = json5::from_str::<ParameterConfig>(&asset_text).ok()
                .expect("Failed to parse parameter config");
            if parameter_name == parameter_config.bound_name {
                self.parameters.insert(parameter_name.to_string(), ConfigId::from_uuid(uuid));
                break;
            }
        }
    }

    pub fn flush_tag_id(&mut self, tag_name: &str) {
        self.tags.remove(tag_name);
    }

    pub fn flush_parameter_id(&mut self, parameter_name: &str) {
        self.parameters.remove(parameter_name);
    }

    pub fn get_tag_id(&mut self, asset_db: &AssetDb, tag_name: &str) -> Option<ConfigId<TagConfig>> {
        if !self.tags.contains_key(tag_name) {
            self.resolve_tag_id(asset_db, tag_name);
        }
        self.tags.get(tag_name).copied()
    }
    pub fn get_parameter_id(&mut self, asset_db: &AssetDb, parameter_name: &str) -> Option<ConfigId<ParameterConfig>> {
        if !self.parameters.contains_key(parameter_name) {
            self.resolve_parameter_id(asset_db, parameter_name);
        }
        self.parameters.get(parameter_name).copied()
    }
}


#[derive(Clone, Debug)]
pub struct ParsedExpressionParameter {
    node: ExpressionParameterNode,
    errors: String,
}

impl ParsedExpressionParameter {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn into_compiled(self) -> CompiledExpressionParameterNode {
        let ParsedExpressionParameter { node, errors } = self;
        if errors.is_empty() {
            CompiledExpressionParameterNode::Ok(node)
        } else {
            CompiledExpressionParameterNode::Error { compile_error: errors }
        }
    }
}

impl ParameterOperator {
    fn from_prefix_token(token: &str) -> Option<Self> {
        match token {
            "+" => Some(Self::Plus),
            "-" => Some(Self::Minus),
            "*" => Some(Self::Mul),
            "/" => Some(Self::Div),
            "clamp" => Some(Self::Clamp),
            "min" => Some(Self::Min),
            "max" => Some(Self::Max),
            "round" => Some(Self::Round),
            "rand" => Some(Self::Rand),
            _ => None,
        }
    }

    fn prefix_token(self) -> &'static str {
        match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Clamp => "clamp",
            Self::Min => "min",
            Self::Max => "max",
            Self::Round => "round",
            Self::Rand => "rand",
        }
    }
}

#[derive(Clone, Debug)]
enum ExpressionToken {
    LParen { byte: usize },
    RParen { byte: usize },
    Atom { text: String, byte: usize },
    Parameter { name: String, raw: String, byte: usize },
    Tag { name: String, raw: String, byte: usize },
    Invalid { raw: String, reason: String, byte: usize },
}

impl ExpressionToken {
    fn byte(&self) -> usize {
        match self {
            Self::LParen { byte }
            | Self::RParen { byte }
            | Self::Atom { byte, .. }
            | Self::Parameter { byte, .. }
            | Self::Tag { byte, .. }
            | Self::Invalid { byte, .. } => *byte,
        }
    }

    fn raw_text(&self) -> &str {
        match self {
            Self::LParen { .. } => "(",
            Self::RParen { .. } => ")",
            Self::Atom { text, .. } => text,
            Self::Parameter { raw, .. } => raw,
            Self::Tag { raw, .. } => raw,
            Self::Invalid { raw, .. } => raw,
        }
    }
}

struct ExpressionTokenizer<'a> {
    text: &'a str,
    cursor: usize,
}

impl<'a> ExpressionTokenizer<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, cursor: 0 }
    }

    fn peek_char(&self) -> Option<char> {
        self.text[self.cursor..].chars().next()
    }

    fn bump_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.cursor += ch.len_utf8();
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while self.peek_char().is_some_and(char::is_whitespace) {
            self.bump_char();
        }
    }

    fn read_plain_atom(&mut self, start: usize) -> ExpressionToken {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() || matches!(ch, '(' | ')' | '{' | '}' | '[' | ']') {
                break;
            }
            self.bump_char();
        }

        ExpressionToken::Atom {
            text: self.text[start..self.cursor].to_string(),
            byte: start,
        }
    }

    fn read_braced_name(
        &mut self,
        start: usize,
        close: char,
        make_token: impl FnOnce(String, String, usize) -> ExpressionToken,
        kind_name: &'static str,
    ) -> ExpressionToken {
        // consume opening bracket
        self.bump_char();
        let name_start = self.cursor;

        while let Some(ch) = self.peek_char() {
            if ch == close {
                let name = self.text[name_start..self.cursor].to_string();
                self.bump_char();
                let raw = self.text[start..self.cursor].to_string();

                if name.is_empty() {
                    return ExpressionToken::Invalid {
                        raw,
                        reason: format!("пустое имя {}", kind_name),
                        byte: start,
                    };
                }

                return make_token(name, raw, start);
            }

            if ch.is_whitespace() || matches!(ch, '(' | ')' | '{' | '[' | '}' | ']') {
                return ExpressionToken::Invalid {
                    raw: self.text[start..self.cursor].to_string(),
                    reason: format!("незавершённая ссылка на {}", kind_name),
                    byte: start,
                };
            }

            self.bump_char();
        }

        ExpressionToken::Invalid {
            raw: self.text[start..self.cursor].to_string(),
            reason: format!("незавершённая ссылка на {}", kind_name),
            byte: start,
        }
    }
}

impl Iterator for ExpressionTokenizer<'_> {
    type Item = ExpressionToken;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let start = self.cursor;
        let ch = self.peek_char()?;

        match ch {
            '(' => {
                self.bump_char();
                Some(ExpressionToken::LParen { byte: start })
            }
            ')' => {
                self.bump_char();
                Some(ExpressionToken::RParen { byte: start })
            }
            '{' => Some(self.read_braced_name(
                start,
                '}',
                |name, raw, byte| ExpressionToken::Parameter { name, raw, byte },
                "черту",
            )),
            '[' => Some(self.read_braced_name(
                start,
                ']',
                |name, raw, byte| ExpressionToken::Tag { name, raw, byte },
                "лычку",
            )),
            '}' | ']' => {
                self.bump_char();
                Some(ExpressionToken::Invalid {
                    raw: ch.to_string(),
                    reason: "закрывающая скобка без открывающей".to_string(),
                    byte: start,
                })
            }
            _ => Some(self.read_plain_atom(start)),
        }
    }
}

struct ExpressionFrame {
    operator: Option<ParameterOperator>,
    args: Vec<ExpressionParameterNode>,
    open_byte: usize,
}

impl ExpressionFrame {
    fn new(open_byte: usize) -> Self {
        Self {
            operator: None,
            args: Vec::new(),
            open_byte,
        }
    }
}

fn push_parse_error(errors: &mut String, message: impl AsRef<str>) {
    if !errors.is_empty() {
        errors.push('\n');
    }
    errors.push_str(message.as_ref());
}

fn escaped_token(token: &str) -> String {
    token.escape_debug().to_string()
}

fn validate_operator_arity(
    operator: ParameterOperator,
    actual_arity: usize,
    errors: &mut String,
) {
    let expected = match operator {
        ParameterOperator::Plus | ParameterOperator::Minus if actual_arity == 0 => {
            Some("ожидался хотя бы 1 аргумент")
        }
        ParameterOperator::Mul | ParameterOperator::Div if actual_arity < 2 => {
            Some("ожидалось хотя бы 2 аргумента")
        }
        ParameterOperator::Min | ParameterOperator::Max if actual_arity != 2 => {
            Some("ожидалось ровно 2 аргумента")
        }
        ParameterOperator::Clamp if actual_arity != 3 => {
            Some("ожидалось ровно 3 аргумента")
        }
        ParameterOperator::Round if actual_arity != 1 => {
            Some("ожидался ровно 1 аргумент")
        }
        ParameterOperator::Rand if actual_arity != 2 => {
            Some("ожидалось ровно 2 аргумента")
        }
        _ => None,
    };

    if let Some(expected) = expected {
        push_parse_error(
            errors,
            format!(
                "Некорректная арность оператора `{}`: {}, получено {}.",
                operator.prefix_token(),
                expected,
                actual_arity,
            ),
        );
    }
}

fn short_tail_from(source: &str, byte: usize) -> String {
    const LIMIT: usize = 80;

    let tail = source[byte..].trim();
    let mut result = String::new();

    for ch in tail.chars().take(LIMIT) {
        result.push(ch);
    }

    if tail.chars().count() > LIMIT {
        result.push_str("...");
    }

    escaped_token(&result)
}

fn receive_expression_node(
    node: ExpressionParameterNode,
    stack: &mut Vec<ExpressionFrame>,
    root: &mut Option<ExpressionParameterNode>,
    errors: &mut String,
) {
    if let Some(parent) = stack.last_mut() {
        if parent.operator.is_some() {
            parent.args.push(node);
        } else {
            push_parse_error(
                errors,
                format!(
                    "В выражении, начатом на байте {}, ожидался оператор; вложенное выражение проигнорировано.",
                    parent.open_byte
                ),
            );
        }
    } else if root.is_none() {
        *root = Some(node);
    }
}

fn close_expression_frame(
    stack: &mut Vec<ExpressionFrame>,
    root: &mut Option<ExpressionParameterNode>,
    errors: &mut String,
    close_byte: usize,
) {
    let Some(frame) = stack.pop() else {
        push_parse_error(
            errors,
            format!("Некорректный токен `)` на байте {}: закрывающая скобка без открывающей.", close_byte),
        );
        return;
    };

    let Some(operator) = frame.operator else {
        push_parse_error(
            errors,
            format!("В выражении, начатом на байте {}, отсутствует оператор.", frame.open_byte),
        );
        return;
    };

    validate_operator_arity(operator, frame.args.len(), errors);

    receive_expression_node(
        ExpressionParameterNode::Operator(operator, frame.args),
        stack,
        root,
        errors,
    );
}

fn collapse_unfinished_frames(
    stack: &mut Vec<ExpressionFrame>,
    root: &mut Option<ExpressionParameterNode>,
    errors: &mut String,
) {
    while let Some(frame) = stack.pop() {
        push_parse_error(
            errors,
            format!("Незавершённое выражение: не закрыта `(`, открытая на байте {}.", frame.open_byte),
        );

        let Some(operator) = frame.operator else {
            push_parse_error(
                errors,
                format!("В незавершённом выражении, начатом на байте {}, отсутствует оператор.", frame.open_byte),
            );
            continue;
        };

        validate_operator_arity(operator, frame.args.len(), errors);

        receive_expression_node(
            ExpressionParameterNode::Operator(operator, frame.args),
            stack,
            root,
            errors,
        );
    }
}

fn resolve_parameter_reference(
    asset_db: &AssetDb,
    cache: &mut ExpressionParameterIdCache,
    name: &str,
    byte: usize,
    errors: &mut String,
) -> ConfigId<ParameterConfig> {
    if let Some(id) = cache.get_parameter_id(asset_db, name) {
        id
    } else {
        push_parse_error(
            errors,
            format!("Неизвестная черта `{}` на байте {}.", escaped_token(name), byte),
        );
        ConfigId::INVALID
    }
}

fn resolve_tag_reference(
    asset_db: &AssetDb,
    cache: &mut ExpressionParameterIdCache,
    name: &str,
    byte: usize,
    errors: &mut String,
) -> ConfigId<TagConfig> {
    if let Some(id) = cache.get_tag_id(asset_db, name) {
        id
    } else {
        push_parse_error(
            errors,
            format!("Неизвестная лычка `{}` на байте {}.", escaped_token(name), byte),
        );
        ConfigId::INVALID
    }
}

fn value_token_to_node(
    asset_db: &AssetDb,
    token: ExpressionToken,
    cache: &mut ExpressionParameterIdCache,
    errors: &mut String,
) -> Option<ExpressionParameterNode> {
    match token {
        ExpressionToken::Atom { text, byte } => match text.parse::<f32>() {
            Ok(value) => Some(ExpressionParameterNode::Constant(value)),
            Err(_) => {
                push_parse_error(
                    errors,
                    format!("Некорректный токен `{}` на байте {}: ожидалось число, ссылка или s-expression.", escaped_token(&text), byte),
                );
                None
            }
        },
        ExpressionToken::Parameter { name, byte, .. } => Some(ExpressionParameterNode::ParameterValue(
            resolve_parameter_reference(asset_db, cache, &name, byte, errors),
        )),
        ExpressionToken::Tag { name, byte, .. } => Some(ExpressionParameterNode::TagCount(
            resolve_tag_reference(asset_db, cache, &name, byte, errors),
        )),
        ExpressionToken::Invalid { raw, reason, byte } => {
            push_parse_error(
                errors,
                format!("Некорректный токен `{}` на байте {}: {}.", escaped_token(&raw), byte, reason),
            );
            None
        }
        ExpressionToken::LParen { .. } | ExpressionToken::RParen { .. } => None,
    }
}

/// Parses an expression based on prefix s-expressions:
///
/// - `{hp}`: parameter value;
/// - `[poisoned]`: tag count;
/// - `123.0`: constant;
/// - `(+ 123.0 {hp} [poisoned] (* 2.0 6.0))`: operator expression.
///
/// The parser intentionally tries to recover after errors:
/// unknown names become `ConfigId::INVALID`, invalid tokens are skipped,
/// unfinished operator expressions are collapsed into partial AST nodes.
pub fn parse_expression_parameter(
    asset_db: &AssetDb,
    source: &str,
    cache: &mut ExpressionParameterIdCache,
) -> ParsedExpressionParameter {
    let mut stack = Vec::<ExpressionFrame>::new();
    let mut root = None::<ExpressionParameterNode>;
    let mut errors = String::new();
    let mut reported_trailing_text = false;

    for token in ExpressionTokenizer::new(source) {
        if root.is_some() && stack.is_empty() {
            if !reported_trailing_text {
                push_parse_error(
                    &mut errors,
                    format!(
                        "После завершённого выражения остался лишний текст, начиная с байта {}: `{}`.",
                        token.byte(),
                        short_tail_from(source, token.byte())
                    ),
                );
                reported_trailing_text = true;
            }

            if let ExpressionToken::Invalid { raw, reason, byte } = token {
                push_parse_error(
                    &mut errors,
                    format!("Некорректный токен `{}` на байте {}: {}.", escaped_token(&raw), byte, reason),
                );
            }

            continue;
        }

        match token {
            ExpressionToken::LParen { byte } => {
                stack.push(ExpressionFrame::new(byte));
            }
            ExpressionToken::RParen { byte } => {
                close_expression_frame(&mut stack, &mut root, &mut errors, byte);
            }
            ExpressionToken::Atom { text, byte } if stack.last().is_some_and(|frame| frame.operator.is_none()) => {
                if let Some(operator) = ParameterOperator::from_prefix_token(&text) {
                    if let Some(frame) = stack.last_mut() {
                        frame.operator = Some(operator);
                    }
                } else {
                    push_parse_error(
                        &mut errors,
                        format!(
                            "Некорректный токен `{}` на байте {}: после `(` ожидался оператор.",
                            escaped_token(&text),
                            byte
                        ),
                    );
                }
            }
            token if stack.last().is_some_and(|frame| frame.operator.is_none()) => {
                push_parse_error(
                    &mut errors,
                    format!(
                        "Некорректный токен `{}` на байте {}: после `(` ожидался оператор.",
                        escaped_token(token.raw_text()),
                        token.byte()
                    ),
                );
            }
            token => {
                if let Some(node) = value_token_to_node(asset_db, token, cache, &mut errors) {
                    receive_expression_node(node, &mut stack, &mut root, &mut errors);
                }
            }
        }
    }

    collapse_unfinished_frames(&mut stack, &mut root, &mut errors);

    let node = match root {
        Some(node) => node,
        None => {
            push_parse_error(
                &mut errors,
                "Выражение пустое или не содержит ни одного разбираемого узла.",
            );
            ExpressionParameterNode::default()
        }
    };

    ParsedExpressionParameter { node, errors }
}

pub fn compile_expression_parameter(
    asset_db: &AssetDb,
    source: &str,
    cache: &mut ExpressionParameterIdCache,
) -> CompiledExpressionParameterNode {
    parse_expression_parameter(asset_db, source, cache).into_compiled()
}

impl ParameterConfig {
    pub fn compile_expression(&mut self, asset_db: &AssetDb, cache: &mut ExpressionParameterIdCache) {
        self.compiled_expression = match &self.parameter_type {
            ParameterType::Constant => CompiledExpressionParameterNode::None,
            ParameterType::Expression(source) => compile_expression_parameter(asset_db, source, cache),
        };
    }

    pub fn compiled_expression(&self) -> &CompiledExpressionParameterNode {
        &self.compiled_expression
    }
}

#[cfg(test)]
mod tests {
    use crate::assets::dummy_asset_db;
    use super::*;
    use super::CompiledExpressionParameterNode::*;
    use super::ParameterOperator::*;
    use super::ExpressionParameterNode::*;

    fn test_damage_tag() -> ConfigId<TagConfig> {
        ConfigId::from_uuid(uuid::Uuid::from_bytes([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1]))
    }
    fn test_health_grown_tag() -> ConfigId<TagConfig> {
        ConfigId::from_uuid(uuid::Uuid::from_bytes([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2]))
    }

    fn test_base_hp_parameter() -> ConfigId<ParameterConfig> {
        ConfigId::from_uuid(uuid::Uuid::from_bytes([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3]))
    }

    fn test_hp_growth_multiplier_parameter() -> ConfigId<ParameterConfig> {
        ConfigId::from_uuid(uuid::Uuid::from_bytes([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4]))
    }

    fn parameter_cache_for_tests() -> ExpressionParameterIdCache {
        let mut tags = HashMap::new();
        let mut parameters = HashMap::new();

        tags.insert(
            "урон".to_string(),
            test_damage_tag()
        );
        tags.insert(
            "здоровье_развил".to_string(),
            test_health_grown_tag()
        );

        parameters.insert(
            "базовое_здоровье".to_string(),
            test_base_hp_parameter()
        );
        parameters.insert(
            "здоровье_прирост".to_string(),
            test_hp_growth_multiplier_parameter()
        );

        ExpressionParameterIdCache {
            tags,
            parameters,
            is_test: true
        }
    }

    fn compile_for_tests(asset_db: &AssetDb, source: &str) -> CompiledExpressionParameterNode {
        let mut cache = parameter_cache_for_tests();
        compile_expression_parameter(&asset_db, source, &mut cache)
    }

    fn parse_for_tests(asset_db: &AssetDb, source: &str) -> ParsedExpressionParameter {
        let mut cache = parameter_cache_for_tests();
        parse_expression_parameter(&asset_db, source, &mut cache)
    }

    fn assert_error_contains(asset_db: &AssetDb, source: &str, expected_part: &str) {
        match compile_for_tests(asset_db, source) {
            Error { compile_error } => {
                assert!(
                    compile_error.contains(expected_part),
                    "source: {source}, expected error part: {expected_part}, actual error: {compile_error}",
                );
            }
            compiled => panic!("expected compile error for `{source}`, got {compiled:?}"),
        }
    }

    fn assert_operator_arity_is_ok(
        asset_db: &AssetDb,
        source: &str,
        expected_operator: ParameterOperator,
        expected_arity: usize,
    ) {
        match compile_for_tests(asset_db, source) {
            Ok(Operator(operator, args)) => {
                assert_eq!(expected_operator, operator, "source: {source}");
                assert_eq!(expected_arity, args.len(), "source: {source}");
            }
            compiled => panic!("expected valid operator expression for `{source}`, got {compiled:?}"),
        }
    }

    fn assert_operator_arity_is_error(
        asset_db: &AssetDb,
        source: &str,
        operator_token: &str,
        actual_arity: usize
    ) {
        match compile_for_tests(asset_db, source) {
            Error { compile_error } => {
                assert!(
                    compile_error.contains("Некорректная арность оператора"),
                    "source: {source}, error: {compile_error}",
                );
                assert!(
                    compile_error.contains(&format!("оператора `{operator_token}`")),
                    "source: {source}, error: {compile_error}",
                );
                assert!(
                    compile_error.contains(&format!("получено {actual_arity}")),
                    "source: {source}, error: {compile_error}",
                );
            }
            compiled => panic!("expected arity error for `{source}`, got {compiled:?}"),
        }
    }

    #[test]
    fn test_parsing() {
        let source = "(- (+ {базовое_здоровье} (* [здоровье_развил] {здоровье_прирост})) [урон])";
        let mut cache = parameter_cache_for_tests();
        let mut parameter_config = ParameterConfig::default();
        parameter_config.parameter_type = ParameterType::Expression(source.to_string());
        let asset_db = dummy_asset_db();
        parameter_config.compile_expression(&asset_db, &mut cache);
        assert_eq!(
            &Ok(
                Operator(
                    Minus,
                    vec![
                        Operator(
                            Plus,
                            vec![
                                ParameterValue(test_base_hp_parameter()),
                                Operator(
                                    Mul,
                                    vec![
                                        TagCount(test_health_grown_tag()),
                                        ParameterValue(test_hp_growth_multiplier_parameter()),
                                    ]
                                )
                            ]
                        ),
                        TagCount(test_damage_tag())
                    ]
                )
            ),
            parameter_config.compiled_expression()
        )
    }

    #[test]
    fn test_plus_and_minus_arity() {
        let asset_db = dummy_asset_db();
        for (source, operator, arity) in [
            ("(+ 1)", Plus, 1),
            ("(+ 1 2)", Plus, 2),
            ("(+ 1 2 3)", Plus, 3),
            ("(- 1)", Minus, 1),
            ("(- 1 2)", Minus, 2),
            ("(- 1 2 3)", Minus, 3),
        ] {
            assert_operator_arity_is_ok(&asset_db, source, operator, arity);
        }

        assert_operator_arity_is_error(&asset_db, "(+)", "+", 0);
        assert_operator_arity_is_error(&asset_db, "(-)", "-", 0);
    }

    #[test]
    fn test_mul_and_div_arity() {
        let asset_db = dummy_asset_db();

        for (source, operator, arity) in [
            ("(* 1 2)", Mul, 2),
            ("(* 1 2 3)", Mul, 3),
            ("(/ 1 2)", Div, 2),
            ("(/ 1 2 3)", Div, 3),
        ] {
            assert_operator_arity_is_ok(&asset_db, source, operator, arity);
        }

        for (source, operator_token, arity) in [
            ("(*)", "*", 0),
            ("(* 1)", "*", 1),
            ("(/)", "/", 0),
            ("(/ 1)", "/", 1),
        ] {
            assert_operator_arity_is_error(&asset_db, source, operator_token, arity);
        }
    }

    #[test]
    fn test_min_and_max_arity() {
        let asset_db = dummy_asset_db();

        assert_operator_arity_is_ok(&asset_db, "(min 1 2)", Min, 2);
        assert_operator_arity_is_ok(&asset_db, "(max 1 2)", Max, 2);

        for (source, operator_token, arity) in [
            ("(min)", "min", 0),
            ("(min 1)", "min", 1),
            ("(min 1 2 3)", "min", 3),
            ("(max)", "max", 0),
            ("(max 1)", "max", 1),
            ("(max 1 2 3)", "max", 3),
        ] {
            assert_operator_arity_is_error(&asset_db, source, operator_token, arity);
        }
    }

    #[test]
    fn test_clamp_arity() {
        let asset_db = dummy_asset_db();

        assert_operator_arity_is_ok(&asset_db, "(clamp 1 2 3)", Clamp, 3);

        for (source, arity) in [
            ("(clamp)", 0),
            ("(clamp 1)", 1),
            ("(clamp 1 2)", 2),
            ("(clamp 1 2 3 4)", 4),
        ] {
            assert_operator_arity_is_error(&asset_db, source, "clamp", arity);
        }
    }

    #[test]
    fn test_round_arity() {
        let asset_db = dummy_asset_db();

        assert_operator_arity_is_ok(&asset_db, "(round 1)", Round, 1);

        for (source, arity) in [
            ("(round)", 0),
            ("(round 1 2)", 2),
        ] {
            assert_operator_arity_is_error(&asset_db, source, "round", arity);
        }
    }

    #[test]
    fn test_rand_arity() {
        let asset_db = dummy_asset_db();

        assert_operator_arity_is_ok(&asset_db, "(rand 1 2)", Rand, 2);

        for (source, arity) in [
            ("(rand)", 0),
            ("(rand 1)", 1),
            ("(rand 1 2 3)", 3),
        ] {
            assert_operator_arity_is_error(&asset_db, source, "rand", arity);
        }
    }

    #[test]
    fn test_unknown_parameter_name_is_error_and_uses_invalid_id() {
        let asset_db = dummy_asset_db();
        let source = "(+ {неизвестная_черта} 1)";
        let parsed = parse_for_tests(&asset_db, source);

        assert!(parsed.has_errors(), "source: {source}");
        assert!(
            parsed.errors.contains("Неизвестная черта `неизвестная_черта`"),
            "source: {source}, error: {}",
            parsed.errors,
        );
        assert_eq!(
            Operator(
                Plus,
                vec![
                    ParameterValue(ConfigId::INVALID),
                    Constant(1.0),
                ],
            ),
            parsed.node,
        );

        assert_error_contains(&asset_db, source, "Неизвестная черта `неизвестная_черта`");
    }

    #[test]
    fn test_unknown_tag_name_is_error_and_uses_invalid_id() {
        let asset_db = dummy_asset_db();
        let source = "(+ [неизвестная_лычка] 1)";
        let parsed = parse_for_tests(&asset_db, source);

        assert!(parsed.has_errors(), "source: {source}");
        assert!(
            parsed.errors.contains("Неизвестная лычка `неизвестная_лычка`"),
            "source: {source}, error: {}",
            parsed.errors,
        );
        assert_eq!(
            Operator(
                Plus,
                vec![
                    TagCount(ConfigId::INVALID),
                    Constant(1.0),
                ],
            ),
            parsed.node,
        );

        assert_error_contains(&asset_db, source, "Неизвестная лычка `неизвестная_лычка`");
    }

    #[test]
    fn test_invalid_atom_tokens_are_skipped_and_parser_continues() {
        let asset_db = dummy_asset_db();
        let source = "(+ 1 мусор 2 @ 3)";
        let parsed = parse_for_tests(&asset_db, source);

        assert!(parsed.has_errors(), "source: {source}");
        assert!(
            parsed.errors.contains("Некорректный токен `мусор`"),
            "source: {source}, error: {}",
            parsed.errors,
        );
        assert!(
            parsed.errors.contains("Некорректный токен `@`"),
            "source: {source}, error: {}",
            parsed.errors,
        );
        assert_eq!(
            Operator(
                Plus,
                vec![
                    Constant(1.0),
                    Constant(2.0),
                    Constant(3.0),
                ],
            ),
            parsed.node,
        );
    }

    #[test]
    fn test_invalid_reference_tokens_are_skipped_and_parser_continues() {
        let asset_db = dummy_asset_db();
        let source = "(+ {битая 1 [ 2 3)";
        let parsed = parse_for_tests(&asset_db, source);

        assert!(parsed.has_errors(), "source: {source}");
        assert!(
            parsed.errors.contains("Некорректный токен `{битая`"),
            "source: {source}, error: {}",
            parsed.errors,
        );
        assert!(
            parsed.errors.contains("Некорректный токен `[`"),
            "source: {source}, error: {}",
            parsed.errors,
        );
        assert_eq!(
            Operator(
                Plus,
                vec![
                    Constant(1.0),
                    Constant(2.0),
                    Constant(3.0),
                ],
            ),
            parsed.node,
        );
    }

    #[test]
    fn test_unfinished_expression_is_error_but_partial_ast_is_returned() {
        let asset_db = dummy_asset_db();
        let source = "(* 2 (+ 3 4)";
        let parsed = parse_for_tests(&asset_db, source);

        assert!(parsed.has_errors(), "source: {source}");
        assert!(
            parsed.errors.contains("Незавершённое выражение"),
            "source: {source}, error: {}",
            parsed.errors,
        );
        assert_eq!(
            Operator(
                Mul,
                vec![
                    Constant(2.0),
                    Operator(
                        Plus,
                        vec![
                            Constant(3.0),
                            Constant(4.0),
                        ],
                    ),
                ],
            ),
            parsed.node,
        );
    }

    #[test]
    fn test_trailing_text_after_complete_expression_is_error() {
        let asset_db = dummy_asset_db();
        let source = "(+ 1 2) (* 3 4)";
        let parsed = parse_for_tests(&asset_db, source);

        assert!(parsed.has_errors(), "source: {source}");
        assert!(
            parsed.errors.contains("После завершённого выражения остался лишний текст"),
            "source: {source}, error: {}",
            parsed.errors,
        );
        assert_eq!(
            Operator(
                Plus,
                vec![
                    Constant(1.0),
                    Constant(2.0),
                ],
            ),
            parsed.node,
        );
    }

}
