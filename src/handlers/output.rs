use crate::Trayle;
use smithay::wayland::output::OutputHandler;

smithay::delegate_output!(Trayle);

/// required for [`Output::create_global`]
///
/// in case of tty backend, when connector connected
///
/// [`Output::create_global`]: smithay::output::Output::create_global
impl OutputHandler for Trayle { }

