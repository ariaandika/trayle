use zwp_linux_dmabuf_v1::{*, Destroy as DmabufDestroy};
use zwp_linux_buffer_params_v1::{*, Destroy as BufferParamsDestroy};
use zwp_linux_dmabuf_feedback_v1::Destroy as FeedbackDestroy;

use crate::compositor::prelude::*;

// ===== zwp_linux_dmabuf_v1 =====

impl MessageHandler<DmabufDestroy> for Compositor {
    fn handle(&mut self, _msg: Msg<DmabufDestroy>, _client: &mut ClientMut) -> Todo<DmabufDestroy> {
        Todo::new()
    }
}

impl MessageHandler<CreateParams> for Compositor {
    fn handle(&mut self, _msg: Msg<CreateParams>, _client: &mut ClientMut) -> Todo<CreateParams> {
        Todo::new()
    }
}

impl MessageHandler<GetDefaultFeedback> for Compositor {
    fn handle(
        &mut self,
        _msg: Msg<GetDefaultFeedback>,
        _client: &mut ClientMut,
    ) -> Todo<GetDefaultFeedback> {
        Todo::new()
    }
}

impl MessageHandler<GetSurfaceFeedback> for Compositor {
    fn handle(
        &mut self,
        _msg: Msg<GetSurfaceFeedback>,
        _client: &mut ClientMut,
    ) -> Todo<GetSurfaceFeedback> {
        Todo::new()
    }
}

// ===== zwp_linux_buffer_params_v1 =====

impl MessageHandler<BufferParamsDestroy> for Compositor {
    fn handle(
        &mut self,
        _msg: Msg<BufferParamsDestroy>,
        _client: &mut ClientMut,
    ) -> Todo<BufferParamsDestroy> {
        Todo::new()
    }
}

impl MessageHandler<Add> for Compositor {
    fn handle(&mut self, _msg: Msg<Add>, _client: &mut ClientMut) -> Todo<Add> {
        Todo::new()
    }
}

impl MessageHandler<Create> for Compositor {
    fn handle(&mut self, _msg: Msg<Create>, _client: &mut ClientMut) -> Todo<Create> {
        Todo::new()
    }
}

impl MessageHandler<CreateImmed> for Compositor {
    fn handle(&mut self, _msg: Msg<CreateImmed>, _client: &mut ClientMut) -> Todo<CreateImmed> {
        Todo::new()
    }
}

// ===== zwp_linux_dmabuf_feedback_v1 =====

impl MessageHandler<FeedbackDestroy> for Compositor {
    fn handle(&mut self, _msg: Msg<FeedbackDestroy>, _client: &mut ClientMut) -> Todo<FeedbackDestroy> {
        Todo::new()
    }
}
