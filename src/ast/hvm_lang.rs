use super::{DefId, Name};
use bimap::{BiHashMap, Overwritten};
use itertools::Itertools;
use std::{
  cell::RefCell,
  fmt::{self, Debug},
};

#[derive(Debug, Clone, Default)]
pub struct DefNames {
  map: RefCell<BiHashMap<DefId, Name>>,
  id_count: RefCell<DefId>,
}

#[derive(Debug, Clone, Default)]
pub struct DefinitionBook {
  pub def_names: DefNames,
  pub defs: Vec<Definition>,
}

#[derive(Debug, Clone)]
pub struct Definition {
  pub def_id: DefId,
  pub rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
pub struct Rule {
  pub def_id: DefId,
  pub pats: Vec<Pattern>,
  pub body: Term,
}

#[derive(Debug, Clone)]
pub enum Pattern {
  Ctr(Name, Vec<Pattern>),
  Var(Option<Name>),
  Num(u32),
}

#[derive(Debug, Clone)]
pub enum Term {
  Lam {
    nam: Option<Name>,
    bod: Box<Term>,
  },
  Var {
    nam: Name,
  },
  /// Like a scopeless lambda, where the variable can occur outside the body
  Chn {
    nam: Name,
    bod: Box<Term>,
  },
  /// The use of a Channel variable.
  Lnk {
    nam: Name,
  },
  Let {
    nam: Name,
    val: Box<Term>,
    nxt: Box<Term>,
  },
  Ref {
    def_id: DefId,
  },
  App {
    fun: Box<Term>,
    arg: Box<Term>,
  },
  If {
    cond: Box<Term>,
    then: Box<Term>,
    els_: Box<Term>,
  },
  Dup {
    fst: Option<Name>,
    snd: Option<Name>,
    val: Box<Term>,
    nxt: Box<Term>,
  },
  Sup {
    fst: Box<Term>,
    snd: Box<Term>,
  },
  Era,
  Num {
    val: u32,
  },
  /// A numeric operation between built-in numbers.
  Opx {
    op: Op,
    fst: Box<Term>,
    snd: Box<Term>,
  },
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
  ADD,
  SUB,
  MUL,
  DIV,
  MOD,
  EQ,
  NE,
  LT,
  GT,
  AND,
  OR,
  XOR,
  NOT,
  LSH,
  RSH,
}

impl DefinitionBook {
  pub fn new() -> Self {
    Default::default()
  }
}

impl DefNames {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn name(&self, def_id: &DefId) -> Option<Name> {
    self.map.borrow().get_by_left(def_id).cloned()
  }

  pub fn def_id(&self, name: &Name) -> Option<DefId> {
    self.map.borrow().get_by_right(name).copied()
  }

  pub fn contains_name(&self, name: &Name) -> bool {
    self.map.borrow().contains_right(name)
  }

  pub fn contains_def_id(&self, def_id: &DefId) -> bool {
    self.map.borrow().contains_left(def_id)
  }

  pub fn insert(&self, name: Name) -> DefId {
    let def_id = {
      let mut id_count = self.id_count.borrow_mut();
      let def_id = *id_count;
      id_count.0 += 1;
      def_id
    };

    match self.map.borrow_mut().insert(def_id, name) {
      Overwritten::Neither => def_id,
      _ => todo!("Overwritting name-id pairs not supported"),
    }
  }
}

impl Term {
  pub fn to_string(&self, def_names: &DefNames) -> String {
    match self {
      Term::Lam { nam, bod } => {
        format!("λ{} {}", nam.clone().unwrap_or(Name::new("*")), bod.to_string(def_names))
      }
      Term::Var { nam } => format!("{nam}"),
      Term::Chn { nam, bod } => format!("λ${} {}", nam, bod.to_string(def_names)),
      Term::Lnk { nam } => format!("${nam}"),
      Term::Let { nam, val, nxt } => {
        format!("let {} = {}; {}", nam, val.to_string(def_names), nxt.to_string(def_names))
      }
      Term::Ref { def_id } => format!("{}", def_names.name(def_id).unwrap()),
      Term::App { fun, arg } => format!("({} {})", fun.to_string(def_names), arg.to_string(def_names)),
      Term::If { cond, then, els_ } => {
        format!(
          "if {} then {} else {}",
          cond.to_string(def_names),
          then.to_string(def_names),
          els_.to_string(def_names)
        )
      }
      Term::Dup { fst, snd, val, nxt } => format!(
        "dup {} {} = {}; {}",
        fst.as_ref().map(|x| x.as_str()).unwrap_or("*"),
        snd.as_ref().map(|x| x.as_str()).unwrap_or("*"),
        val.to_string(def_names),
        nxt.to_string(def_names)
      ),
      Term::Sup { fst, snd } => format!("{{{} {}}}", fst.to_string(def_names), snd.to_string(def_names)),
      Term::Era => "*".to_string(),
      Term::Num { val } => format!("{val}"),
      Term::Opx { op, fst, snd } => {
        format!("({} {} {})", op, fst.to_string(def_names), snd.to_string(def_names))
      }
    }
  }

  /// Make a call term by folding args around a called function term with applications.
  pub fn call(called: Term, args: impl IntoIterator<Item = Term>) -> Self {
    args.into_iter().fold(called, |acc, arg| Term::App { fun: Box::new(acc), arg: Box::new(arg) })
  }

  /// Substitute the occurences of a variable in a term with the given term.
  pub fn subst(&mut self, from: &Name, to: &Term) {
    match self {
      Term::Lam { nam: Some(nam), .. } if nam == from => (),
      Term::Lam { bod, .. } => bod.subst(from, to),
      Term::Var { nam } if nam == from => *self = to.clone(),
      Term::Var { .. } => (),
      // Only substitute scoped variables.
      Term::Chn { bod, .. } => bod.subst(from, to),
      Term::Lnk { .. } => (),
      Term::Let { nam, val, nxt } => {
        val.subst(from, to);
        if nam != from {
          nxt.subst(from, to);
        }
      }
      Term::If { cond, then, els_ } => {
        cond.subst(from, to);
        then.subst(from, to);
        els_.subst(from, to);
      }
      Term::Ref { .. } => (),
      Term::App { fun, arg } => {
        fun.subst(from, to);
        arg.subst(from, to);
      }
      Term::Dup { fst, snd, val, nxt } => {
        val.subst(from, to);
        if fst.as_ref().map_or(true, |fst| fst != from) && snd.as_ref().map_or(true, |snd| snd != from) {
          nxt.subst(from, to);
        }
      }
      Term::Sup { fst, snd } => {
        fst.subst(from, to);
        snd.subst(from, to);
      }
      Term::Era => (),
      Term::Num { .. } => (),
      Term::Opx { fst, snd, .. } => {
        fst.subst(from, to);
        snd.subst(from, to);
      }
    }
  }
}

impl Rule {
  pub fn to_string(&self, def_names: &DefNames) -> String {
    let Rule { def_id, pats, body } = self;
    format!(
      "({}{}) = {}",
      def_names.name(def_id).unwrap(),
      pats.iter().map(|x| format!(" {x}")).join(""),
      body.to_string(def_names)
    )
  }

  pub fn arity(&self) -> usize {
    self.pats.len()
  }
}

impl Definition {
  pub fn to_string(&self, def_names: &DefNames) -> String {
    self.rules.iter().map(|x| x.to_string(def_names)).join("\n")
  }

  pub fn arity(&self) -> usize {
    self.rules[0].arity()
  }
}

impl From<&Pattern> for Term {
  fn from(value: &Pattern) -> Self {
    match value {
      Pattern::Ctr(nam, args) => Term::call(Term::Var { nam: nam.clone() }, args.iter().map(Term::from)),
      Pattern::Var(nam) => Term::Var { nam: Name::new(nam.as_ref().map(|x| x.as_str()).unwrap_or("_")) },
      Pattern::Num(num) => Term::Num { val: *num },
    }
  }
}

impl fmt::Display for DefinitionBook {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.defs.iter().map(|x| x.to_string(&self.def_names)).join("\n\n"))
  }
}

impl fmt::Display for Pattern {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Pattern::Ctr(name, pats) => write!(f, "({}{})", name, pats.iter().map(|p| format!(" {p}")).join("")),
      Pattern::Var(nam) => write!(f, "{}", nam.as_ref().map(|x| x.as_str()).unwrap_or("*")),
      Pattern::Num(num) => write!(f, "{num}"),
    }
  }
}

impl fmt::Display for Op {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Op::ADD => write!(f, "+"),
      Op::SUB => write!(f, "-"),
      Op::MUL => write!(f, "*"),
      Op::DIV => write!(f, "/"),
      Op::MOD => write!(f, "%"),
      Op::EQ => write!(f, "=="),
      Op::NE => write!(f, "!="),
      Op::LT => write!(f, "<"),
      Op::GT => write!(f, ">"),
      Op::AND => write!(f, "&"),
      Op::OR => write!(f, "|"),
      Op::XOR => write!(f, "^"),
      Op::NOT => write!(f, "~"),
      Op::LSH => write!(f, "<<"),
      Op::RSH => write!(f, ">>"),
    }
  }
}
