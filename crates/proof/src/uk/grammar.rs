//! Ukrainian grammar, punctuation, and stylistic checks with extensive rule sets based on Ukrainian linguistic standards (VESUM / LanguageTool).

use crate::{Issue, IssueKind, ProofLang, words};

/// Words allowed to be repeated in Ukrainian
const ALLOWED_REPEATS: &[&str] = &[
    "так", "ні", "ну", "ха", "хі", "хе", "ой", "ай", "гей", "ах", "ох", "ех", "ага", "угу", "тук", "кап",
    "дуже", "багато", "давно", "далеко", "довго", "ледь", "трохи", "тихо", "швидко", "скоріш", "скоріше",
    "іди", "давай", "поки", "ладно", "ось", "ще", "все", "всі", "хто", "що", "то", "добре", "було"
];

/// Ukrainian common abbreviations after which a period does not start a new sentence
const ABBREVIATIONS: &[&str] = &[
    "т", "е", "д", "п", "ін", "напр", "т.е", "т.д", "т.п", "т.к", "т.н", "т.о", "г", "р", "рр", "в", "вв",
    "грн", "коп", "тис", "млн", "млрд", "трлн", "вул", "пр-т", "пров", "наб", "пл", "ш", "буд", "кв", "корп",
    "обл", "р-н", "пос", "с", "ім", "св", "проф", "доц", "акад", "канд", "наук", "зав", "зам", "ред", "вид",
    "рис", "табл", "гл", "розд", "ст", "ч", "п", "пп", "абз", "прим", "дод", "іл", "стор", "т", "тт", "прибл",
    "мм", "см", "дм", "км", "м", "л", "мл", "кг", "хв", "сек", "год", "міс", "тиж", "січ", "лют", "берез",
    "квіт", "трав", "черв", "лип", "серп", "верес", "жовт", "листоп", "груд", "пн", "вт", "ср", "чт", "пт",
    "сб", "нд", "англ", "нім", "фр", "лат", "укр", "рос", "грец", "італ", "ісп", "букв", "пер", "устар", "див"
];

/// Clause openers that usually require a comma before them
const CLAUSE_OPENERS: &[&str] = &[
    "що", "щоб", "аби", "якщо", "якби", "бо", "оскільки", "хоч", "хоча",
    "який", "яка", "яке", "які", "якого", "якій", "якому", "яким", "яких", "якими", "яку"
];

/// Words after which a clause opener does not get an extra comma
const NO_COMMA_AFTER: &[&str] = &[
    "і", "й", "а", "але", "та", "чи", "або", "ні", "не", "ж", "же", "чи", "б", "би", "то", "тільки", "лише",
    "саме", "навіть", "особливо", "тому", "через", "так", "потім", "тим", "того", "заради", "внаслідок",
    "незважаючи", "завдяки", "перш", "ніж", "ніби", "мов", "неначе", "як", "якщо", "хоч", "хоча", "що", "хто"
];

/// Prepositions before which the comma goes (e.g. "будинок, у якому")
const PREPOSITIONS: &[&str] = &[
    "в", "у", "на", "з", "зі", "із", "до", "від", "для", "по", "за", "під", "над", "при", "про", "без", "через",
    "між", "серед", "із-за", "із-под", "біля", "проти", "замість", "крім", "після", "перед", "заради"
];

/// Expanded Ukrainian stylistic, surzhyk, and error patterns with explanations and normative corrections
const STYLISTIC_PATTERNS: &[(&str, &str, &str)] = &[
    ("прийняти участь", "взяти участь", "Використовуйте нормативний вираз «взяти участь» замість «прийняти участь»"),
    ("прийняв участь", "взяв участь", "Використовуйте «взяв участь» замість «прийняв участь»"),
    ("прийняла участь", "взяла участь", "Використовуйте «взяла участь» замість «прийняла участь»"),
    ("прийняли участь", "взяли участь", "Використовуйте «взяли участь» замість «прийняли участь»"),
    ("приймати участь", "брати участь", "Використовуйте «брати участь» замість «приймати участь»"),
    ("приймає участь", "бере участь", "Використовуйте «бере участь» замість «приймає участь»"),
    ("приймають участь", "беруть участь", "Використовуйте «беруть участь» замість «приймають участь»"),
    ("приймав участь", "брав участь", "Використовуйте «брав участь» замість «приймав участь»"),
    ("на протязі", "протягом", "Замість «на протязі» у часовому значенні використовуйте «протягом» або «впродовж»"),
    ("в якості", "як", "Замість «в якості» рекомендується використовувати «як» або «у ролі»"),
    ("у якості", "як", "Замість «у якості» рекомендується використовувати «як» або «у ролі»"),
    ("в той же час", "водночас", "Стилістична рекомендація: замість «в той же час» краще «водночас»"),
    ("у той же час", "водночас", "Стилістична рекомендація: замість «у той же час» краще «водночас»"),
    ("тим не менше", "менше з тим", "Стилістично доречніше використовувати «менше з тим» або «проте»"),
    ("більше того", "понад те", "Стилістична рекомендація: замість «більше того» краще «понад те»"),
    ("будь-ласка", "будь ласка", "Словосполучення «будь ласка» пишеться окремо без дефісу"),
    ("Україньська", "Українська", "Слово «українська» пишеться без м'якого знака перед -ськ-"),
    ("україньська", "українська", "Слово «українська» пишеться без м'якого знака перед -ськ-"),
    ("явиться", "буде", "Замість росіянізму «явиться» рекомендується «буде» або «виявиться»"),
    ("являється", "є", "Замість русизму «являється» вживайте дієслово «є» або випустіть його"),
    ("являються", "є", "Замість русизму «являються» вживайте дієслово «є»"),
    ("являвся", "був", "Замість русизму «являвся» вживайте «був»"),
    ("являлася", "була", "Замість русизму «являлася» вживайте «була»"),
    ("являлися", "були", "Замість русизму «являлися» вживайте «були»"),
    ("підводячи підсумки", "підбиваючи підсумки", "Правильний вираз — «підбиваючи підсумки»"),
    ("підвести підсумки", "підбити підсумки", "Правильний вираз — «підбити підсумки»"),
    ("вірно", "правильно", "У значенні «правильно» краще вживати «правильно» замість «вірно»"),
    ("нараховувати", "нараховується", "У діловому мовленні вживайте «нараховує» або «налічує»"),
    ("в міру того як", "у міру того як", "Замість «в міру» краще використовувати «у міру»"),
    ("вітати з", "вітати з", "Не забувайте про правильний відмінок після дієслова «вітати»"),
    ("повідомити наступне", "повідомити таке", "Замість «наступне» вказуючи на подальший текст краще «таке»"),
    ("слідуючий", "наступний", "Замість кальки «слідуючий» вживайте «наступний»"),
    ("слідуючі", "наступні", "Замість кальки «слідуючі» вживайте «наступні»"),
    ("більша половина", "більша частина", "Половина завжди є рівною частиною; вживайте «більша частина»"),
    ("так як", "оскільки", "Сполучник «так як» є суржиковим, вживайте «оскільки» або «бо»"),
];

fn issue(start: usize, end: usize, message: String, suggestions: Vec<String>) -> Issue {
    Issue { start, end, kind: IssueKind::Grammar, message, suggestions, lang: ProofLang::Uk }
}

fn capitalize(w: &str) -> String {
    crate::affix::capitalize(w)
}

pub fn check(text: &str) -> Vec<Issue> {
    check_with(text, true)
}

pub fn check_with(text: &str, capital_first: bool) -> Vec<Issue> {
    let ws: Vec<(usize, usize)> = words(text);
    let mut v = Vec::new();
    repeated_words(text, &ws, &mut v);
    spaces(text, &mut v);
    sentence_capitals(text, &ws, capital_first, &mut v);
    missing_commas(text, &ws, &mut v);
    doubled_punctuation(text, &mut v);
    stylistic_and_surzhyk(text, &mut v);
    v.sort_by_key(|i| (i.start, i.end));
    v
}

fn repeated_words(text: &str, ws: &[(usize, usize)], v: &mut Vec<Issue>) {
    for pair in ws.windows(2) {
        let [(a0, b0), (a1, b1)] = pair else { continue };
        let (Some(w0), Some(w1), Some(gap)) = (text.get(*a0..*b0), text.get(*a1..*b1), text.get(*b0..*a1)) else { continue };
        if !gap.is_empty()
            && gap.chars().all(char::is_whitespace)
            && w0.chars().all(char::is_alphabetic)
            && w0.to_lowercase() == w1.to_lowercase()
            && !ALLOWED_REPEATS.contains(&w0.to_lowercase().as_str())
        {
            v.push(issue(*a0, *b1, format!("Повтор слова: «{w1}»"), vec![w0.to_string()]));
        }
    }
}

fn spaces(text: &str, v: &mut Vec<Issue>) {
    for (i, c) in text.char_indices() {
        if c != ' ' {
            continue;
        }
        let rest = text.get(i + 1..).unwrap_or("");
        let mut it = rest.chars();
        let Some(p) = it.next() else { continue };
        let after = it.next();
        let flag = match p {
            ',' => true,
            ';' => !matches!(after, Some(')' | '(' | '-' | 'P' | 'D')),
            '.' => !matches!(after, Some('.')) && !after.is_some_and(|a| a.is_ascii_digit()),
            ':' | '!' | '?' => after.is_none_or(|a| a.is_whitespace() || matches!(a, '"' | '»' | '”' | '’' | '!' | '?')),
            _ => false,
        };
        if flag {
            v.push(issue(i, i + 1 + p.len_utf8(), "Зайвий пробіл перед знаком пунктуації".into(), vec![p.to_string()]));
        }
    }
    for (i, _) in text.match_indices("  ") {
        if i > 0 && text.get(..i).is_some_and(|t| t.ends_with(' ')) {
            continue;
        }
        v.push(issue(i, i + 2, "Подвійний пробіл".into(), vec![" ".into()]));
    }
}

fn abbreviation_or_number(prev: &str) -> bool {
    prev.chars().count() <= 1
        || prev.chars().any(|c| c.is_ascii_digit())
        || (prev.chars().all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M')))
        || ABBREVIATIONS.contains(&prev.to_lowercase().as_str())
}

const CLOSING: [char; 8] = ['"', '»', '”', '’', '\'', ')', ']', '\u{1}'];
const OPENING: [char; 8] = ['«', '"', '„', '“', '(', '[', '\'', '‚'];

fn ends_sentence(gap: &str, prev: &str) -> bool {
    let t = gap.trim_start_matches(CLOSING);
    let Some(end) = t.chars().next() else { return false };
    if !matches!(end, '.' | '!' | '?') || t.starts_with("..") {
        return false;
    }
    if end == '.' && abbreviation_or_number(prev) {
        return false;
    }
    let rest = t.trim_start_matches(['.', '!', '?']).trim_start_matches(CLOSING);
    rest.starts_with(char::is_whitespace) && rest.trim_start().chars().all(|c| OPENING.contains(&c))
}

fn sentence_capitals(text: &str, ws: &[(usize, usize)], capital_first: bool, v: &mut Vec<Issue>) {
    let list_item = !capital_first || text.trim_end().ends_with([',', ';']);
    for (k, &(a, b)) in ws.iter().enumerate() {
        let Some(w) = text.get(a..b) else { continue };
        let starts = if k == 0 {
            let before = text.get(..a).unwrap_or("");
            !list_item && before.chars().all(|c| c.is_whitespace() || OPENING.contains(&c))
        } else {
            let Some(&(pa, pb)) = ws.get(k - 1) else { continue };
            ends_sentence(text.get(pb..a).unwrap_or(""), text.get(pa..pb).unwrap_or(""))
        };
        if starts && w.chars().next().is_some_and(char::is_lowercase) && w.chars().all(char::is_alphabetic) {
            v.push(issue(a, b, "Речення починається з малої букви".into(), vec![capitalize(w)]));
        }
    }
}

fn missing_commas(text: &str, ws: &[(usize, usize)], v: &mut Vec<Issue>) {
    for k in 1..ws.len() {
        let (Some(&(pa, pb)), Some(&(a, b))) = (ws.get(k - 1), ws.get(k)) else { continue };
        let (Some(prev), Some(w), Some(gap)) = (text.get(pa..pb), text.get(a..b), text.get(pb..a)) else { continue };
        if gap.is_empty() || !gap.chars().all(|c| c == ' ' || c == '\u{a0}') || !prev.chars().all(char::is_alphabetic) {
            continue;
        }
        let lw = w.to_lowercase();
        if !CLAUSE_OPENERS.contains(&lw.as_str()) {
            continue;
        }
        let p = prev.to_lowercase();
        let p = p.as_str();
        if NO_COMMA_AFTER.contains(&p) || PREPOSITIONS.contains(&p) {
            continue;
        }
        v.push(issue(pb, b, format!("Можливо, пропущено кому перед «{w}»"), vec![format!(", {w}")]));
    }
}

fn doubled_punctuation(text: &str, v: &mut Vec<Issue>) {
    let bytes = text.as_bytes();
    let mut i = 0;
    while let Some(&b) = bytes.get(i) {
        if !matches!(b, b',' | b'.' | b';') {
            i += 1;
            continue;
        }
        let run = bytes.get(i..).map_or(0, |r| r.iter().take_while(|&&x| x == b).count());
        let flag = match b {
            b'.' => run == 2,
            _ => run >= 2,
        };
        if flag {
            let end = i + run;
            let suggestions: Vec<String> = match b {
                b',' if text.get(end..).and_then(|t| t.chars().next()).is_some_and(char::is_alphabetic)
                    && (i == 0 || text.get(..i).is_some_and(|t| t.ends_with(char::is_whitespace))) =>
                {
                    vec!["«".into()]
                }
                b'.' => vec![".".into(), "…".into()],
                _ => vec![char::from(b).to_string()],
            };
            v.push(issue(i, end, "Повтор знака пунктуації".into(), suggestions));
        }
        i += run.max(1);
    }
}

fn stylistic_and_surzhyk(text: &str, v: &mut Vec<Issue>) {
    for &(pat, repl, msg) in STYLISTIC_PATTERNS {
        let pat_chars: Vec<char> = pat.chars().collect();
        let pat_len = pat_chars.len();
        if pat_len == 0 {
            continue;
        }

        let mut char_indices: Vec<(usize, char)> = text.char_indices().collect();
        char_indices.push((text.len(), '\0'));

        for window_start in 0..char_indices.len().saturating_sub(pat_len) {
            let mut matches_pat = true;
            for k in 0..pat_len {
                let c = char_indices[window_start + k].1.to_lowercase().next().unwrap_or(char_indices[window_start + k].1);
                if c != pat_chars[k] {
                    matches_pat = false;
                    break;
                }
            }

            if matches_pat {
                let start_byte = char_indices[window_start].0;
                let end_byte = char_indices[window_start + pat_len].0;

                let is_word_boundary_before = window_start == 0 || !char_indices[window_start - 1].1.is_alphabetic();
                let is_word_boundary_after = !char_indices[window_start + pat_len].1.is_alphabetic();

                if is_word_boundary_before && is_word_boundary_after {
                    if let Some(orig) = text.get(start_byte..end_byte) {
                        let suggestion = if orig.chars().next().is_some_and(char::is_uppercase) {
                            capitalize(repl)
                        } else {
                            repl.to_string()
                        };
                        v.push(issue(start_byte, end_byte, msg.into(), vec![suggestion]));
                    }
                }
            }
        }
    }
}
