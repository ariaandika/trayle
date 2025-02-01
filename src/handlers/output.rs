use crate::Trayle;
use smithay::wayland::output::OutputHandler;


smithay::delegate_output!(@<B: 'static> Trayle<B>);

/// used by [`Output::create_global`]
///
/// [`Output`]: smithay::output::Output
impl<B> OutputHandler for Trayle<B> { }

