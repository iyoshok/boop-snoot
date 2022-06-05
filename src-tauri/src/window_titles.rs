use rand::seq::SliceRandom;

const MOJIS: [&str; 87] = [
    "(* ^ ω ^)",
    "(´ ∀ ` *)",
    "٩(◕‿◕｡)۶",
    "☆*:.｡.o(≧▽≦)o.｡.:*☆",
    "(o^▽^o)",
    "(⌒▽⌒)☆",
    "&lt;(￣︶￣)&gt;",
    "。.:☆*:･'(*⌒―⌒*)))",
    "ヽ(・∀・)ﾉ",
    "(´｡• ω •｡`)",
    "(￣ω￣)",
    "｀;:゛;｀;･(°ε° )",
    "(o･ω･o)",
    "(＠＾◡＾)",
    "ヽ(*・ω・)ﾉ",
    "(o_ _)ﾉ彡☆",
    "(^人^)",
    "(o´▽`o)",
    "(*´▽`*)",
    "｡ﾟ( ﾟ^∀^ﾟ)ﾟ｡",
    "( ´ ω ` )",
    "(((o(*°▽°*)o)))",
    "(≧◡≦)",
    "(o´∀`o)",
    "(´• ω •`)",
    "(＾▽＾)",
    "(⌒ω⌒)",
    "∑d(°∀°d)",
    "╰(▔∀▔)╯",
    "(─‿‿─)",
    "(*^‿^*)",
    "ヽ(o^ ^o)ﾉ",
    "(✯◡✯)",
    "(◕‿◕)",
    "(*≧ω≦*)",
    "(☆▽☆)",
    "(⌒‿⌒)",
    "＼(≧▽≦)／",
    "ヽ(o＾▽＾o)ノ",
    "☆ ～('▽^人)",
    "(*°▽°*)",
    "٩(｡•́‿•̀｡)۶",
    "(✧ω✧)",
    "ヽ(*⌒▽⌒*)ﾉ",
    "(´｡• ᵕ •｡`)",
    "( ´ ▽ ` )",
    "(￣▽￣)",
    "╰(*´︶`*)╯",
    "ヽ(&gt;∀&lt;☆)ノ",
    "o(≧▽≦)o",
    "(☆ω☆)",
    "(っ˘ω˘ς )",
    "＼(￣▽￣)／",
    "(*¯︶¯*)",
    "＼(＾▽＾)／",
    "٩(◕‿◕)۶",
    "(o˘◡˘o)",
    "\\(★ω★)/",
    "\\(^ヮ^)/",
    "(〃＾▽＾〃)",
    "(╯✧▽✧)╯",
    "o(&gt;ω&lt;)o",
    "o( ❛ᴗ❛ )o",
    "｡ﾟ(TヮT)ﾟ｡",
    "( ‾́ ◡ ‾́ )",
    "(ﾉ´ヮ`)ﾉ*: ･ﾟ",
    "(b ᵔ▽ᵔ)b",
    "(๑˃ᴗ˂)ﻭ",
    "(๑˘︶˘๑)",
    "( ˙꒳​˙ )",
    "(*꒦ິ꒳꒦ີ)",
    "°˖✧◝(⁰▿⁰)◜✧˖°",
    "(´･ᴗ･ ` )",
    "(ﾉ◕ヮ◕)ﾉ*:･ﾟ✧",
    "(„• ֊ •„)",
    "(.❛ ᴗ ❛.)",
    "(⁀ᗢ⁀)",
    "(￢‿￢ )",
    "(¬‿¬ )",
    "(*￣▽￣)b",
    "( ˙▿˙ )",
    "(¯▿¯)",
    "( ◕▿◕ )",
    "＼(٥⁀▽⁀ )／",
    "(„• ᴗ •„)",
    "(ᵔ◡ᵔ)",
    "( ´ ▿ ` )"
];

const GREETINGS: [&str; 29] = [
    "Hello",
    "Howdy",
    "Hey",
    "Hi",
    "What's kicking?",
    "Howdy-doody",
    "Hello there",
    "Ahoy",
    "Hiya",
    "‘Ello, gov'nor!",
    "What’s crackin’?",
    "‘Sup",
    "Good morning (or evening?)",
    "Hello, my name is Inigo Montoya",
    "Here's Johnny!",
    "Yo!",
    "Whaddup",
    "‘Ello, mate",
    "Heeey, baaaaaby",
    "Yoooouhoooo",
    "How you doin'?",
    "What's cookin', good lookin'?",
    "Hola",
    "Aloha",
    "Que pasa",
    "Bonjour",
    "Hallo",
    "Ciao",
    "こんにちは"
];

pub fn get_random_window_title() -> String {
    let mut rng = rand::thread_rng();
    format!(
        "{} {} #BOOP",
        GREETINGS.choose(&mut rng).unwrap(),
        MOJIS.choose(&mut rng).unwrap()
    )
}