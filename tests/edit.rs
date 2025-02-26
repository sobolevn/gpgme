use gpgme;

use std::io::prelude::*;

use gpgme::{
    edit::{self, EditInteractionStatus, Editor},
    Error, Result,
};

use self::support::passphrase_cb;

#[macro_use]
mod support;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum TestEditorState {
    Start,
    Fingerprint,
    Expire,
    Valid,
    Uid,
    Primary,
    Quit,
    Save,
}

impl Default for TestEditorState {
    fn default() -> Self {
        TestEditorState::Start
    }
}

struct TestEditor;

impl Editor for TestEditor {
    type State = TestEditorState;

    fn next_state(
        state: Result<Self::State>, status: EditInteractionStatus<'_>, need_response: bool,
    ) -> Result<Self::State> {
        use self::TestEditorState as State;

        println!("[-- Code: {:?}, {:?} --]", status.code, status.args());
        if !need_response {
            return state;
        }

        if status.args() == Ok(edit::PROMPT) {
            match state {
                Ok(State::Start) => Ok(State::Fingerprint),
                Ok(State::Fingerprint) => Ok(State::Expire),
                Ok(State::Valid) => Ok(State::Uid),
                Ok(State::Uid) => Ok(State::Primary),
                Ok(State::Quit) => state,
                Ok(State::Primary) | Err(_) => Ok(State::Quit),
                _ => Err(Error::GENERAL),
            }
        } else if (status.args() == Ok(edit::KEY_VALID)) && (state == Ok(State::Expire)) {
            Ok(State::Valid)
        } else if (status.args() == Ok(edit::CONFIRM_SAVE)) && (state == Ok(State::Quit)) {
            Ok(State::Save)
        } else {
            state.and(Err(Error::GENERAL))
        }
    }

    fn action<W: Write>(&self, state: Self::State, mut out: W) -> Result<()> {
        use self::TestEditorState as State;

        match state {
            State::Fingerprint => out.write_all(b"fpr")?,
            State::Expire => out.write_all(b"expire")?,
            State::Valid => out.write_all(b"0")?,
            State::Uid => out.write_all(b"1")?,
            State::Primary => out.write_all(b"primary")?,
            State::Quit => write!(out, "{}", edit::QUIT)?,
            State::Save => write!(out, "{}", edit::YES)?,
            _ => return Err(Error::GENERAL),
        }
        Ok(())
    }
}

test_case! {
    test_edit(test) {
        test.create_context().with_passphrase_provider(passphrase_cb, |ctx| {
            let key = fail_if_err!(ctx.find_keys(Some("Alpha"))).next().unwrap().unwrap();
            fail_if_err!(ctx.edit_key_with(&key, TestEditor, &mut Vec::new()));
        });
    }
}
