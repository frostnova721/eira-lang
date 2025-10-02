#[repr(usize)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    // Keywords
    Channel, // imports
    Bind,
    Mark,
    Seal,   // variable declarations
    Cast,   // invocation
    Secret, Forge, // access modifiers 
    Spell,  // function
    Tome,   // class
    Sign,   // struct
    Attune, // method pairing for struct
    Fate, Divert, // If/Else like divert if its not a fate
    While,
    True, False,
    Release, // return
    // Maybe, // yeah maybe
    Refers, // inheritance
    Origin, // super
    _Self, // this/self
    Chant, // print
    Alias, // type aliasing
    Glyph, // enum
    Sever, // break
    Flow, // continue
    Weave, // types (more like set of behaviours)

    // Symbols
    SemiColon,
    Colon, ColonColon, QuestionMark, // colon for typing and their pair for the thing(?:)!
    Equal, EqualEqual, // =
    Greater, GreaterEqual, // >
    Less, LessEqual, // <
    Bang, BangEqual, // !
    Minus, Plus, Star, Slash, // arithematic
    Dot,
    Comma,
    ParenLeft, ParenRight,
    BraceLeft, BraceRight,
    And, AndAnd, // &
    Or, OrOr, // |

    Identifier,
    Number,
    String,
    StringStart, // starting "
    StringEnd, // closing " 
    InterpolateStart, // <
    InterpolateEnd, // >

    Error, Eof, // well.. idk
}
