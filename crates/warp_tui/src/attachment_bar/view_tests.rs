use warp::appearance::Appearance;
use warp::tui_export::{AttachmentType, PendingAttachmentSummary};
use warp_localization::LocaleId;
use warpui::EntityIdMap;
use warpui_core::elements::MouseStateHandle;
use warpui_core::elements::tui::{
    TuiBuffer, TuiBufferExt, TuiConstraint, TuiLayoutContext, TuiPaintContext, TuiPaintSurface,
    TuiRect, TuiScreenPosition, TuiSize,
};
use warpui_core::{App, AppContext};

use super::{attachment_binding_description_for_locale, render_attachment_snapshot_for_locale};
use crate::attachment_bar::model::TuiAttachmentSnapshot;

fn render_lines(
    snapshot: TuiAttachmentSnapshot,
    width: u16,
    locale: LocaleId,
    ctx: &AppContext,
) -> Vec<String> {
    let mut element = render_attachment_snapshot_for_locale(
        snapshot,
        false,
        MouseStateHandle::default(),
        MouseStateHandle::default(),
        MouseStateHandle::default(),
        locale,
        ctx,
    );
    let mut rendered_views = EntityIdMap::default();
    let mut layout_ctx = TuiLayoutContext {
        rendered_views: &mut rendered_views,
    };
    let size = element.layout(
        TuiConstraint::loose(TuiSize::new(width, 1)),
        &mut layout_ctx,
        ctx,
    );
    let area = TuiRect::new(0, 0, size.width, size.height);
    let mut buffer = TuiBuffer::empty(area);
    let mut paint_ctx = TuiPaintContext::new(&mut rendered_views);
    let mut surface = TuiPaintSurface::new(&mut buffer);
    element.render(
        TuiScreenPosition::new(i32::from(area.x), i32::from(area.y)),
        &mut surface,
        &mut paint_ctx,
    );
    buffer.to_lines()
}

fn snapshot(file_name: &str, position: usize, count: usize) -> TuiAttachmentSnapshot {
    TuiAttachmentSnapshot {
        selected: Some(PendingAttachmentSummary {
            index: position - 1,
            attachment_type: AttachmentType::Image,
            file_name: file_name.to_owned(),
        }),
        position: Some(position),
        count,
        is_processing: false,
        selected_is_processing: false,
    }
}

#[test]
fn renders_single_attachment_without_carousel_arrows() {
    App::test((), |mut app| async move {
        app.update(|ctx| {
            ctx.add_singleton_model(|_| Appearance::mock());
            let line =
                render_lines(snapshot("screenshot.png", 1, 1), 60, LocaleId::EnUs, ctx).remove(0);
            assert!(line.contains("[image]"));
            assert!(line.contains("screenshot.png"));
            assert!(line.contains("1/1"));
            assert!(line.contains('x'));
            assert!(!line.contains('<'));
            assert!(!line.contains('>'));
        });
    });
}

#[test]
fn renders_carousel_position_and_truncates_at_narrow_width() {
    App::test((), |mut app| async move {
        app.update(|ctx| {
            ctx.add_singleton_model(|_| Appearance::mock());
            let line = render_lines(
                snapshot("a-very-long-screenshot-name.png", 2, 3),
                28,
                LocaleId::EnUs,
                ctx,
            )
            .remove(0);
            assert!(line.contains("[image]"));
            assert!(line.contains("2/3"));
            assert!(line.contains('<'));
            assert!(line.contains('>'));
            assert!(line.contains('x'));
            assert!(line.chars().count() <= 28);
        });
    });
}

#[test]
fn renders_provisional_filename_while_image_is_loading() {
    App::test((), |mut app| async move {
        app.update(|ctx| {
            ctx.add_singleton_model(|_| Appearance::mock());
            let lines = render_lines(
                TuiAttachmentSnapshot {
                    selected: Some(PendingAttachmentSummary {
                        index: 0,
                        attachment_type: AttachmentType::Image,
                        file_name: "clipboard-image.png".to_owned(),
                    }),
                    position: Some(1),
                    count: 1,
                    is_processing: true,
                    selected_is_processing: true,
                },
                60,
                LocaleId::EnUs,
                ctx,
            );
            let line = &lines[0];
            assert!(line.contains("[image]"));
            assert!(line.contains("clipboard-image.png"));
            assert!(line.contains("loading…"));
            assert!(line.contains('x'));

            let lines = render_lines(
                TuiAttachmentSnapshot {
                    selected: Some(PendingAttachmentSummary {
                        index: 1,
                        attachment_type: AttachmentType::Image,
                        file_name: "clipboard-image.png".to_owned(),
                    }),
                    position: Some(2),
                    count: 2,
                    is_processing: true,
                    selected_is_processing: true,
                },
                60,
                LocaleId::EnUs,
                ctx,
            );
            let line = &lines[0];
            assert!(line.contains('<'));
            assert!(line.contains('>'));
            assert!(line.contains('x'));
        });
    });
}

#[test]
fn renders_simplified_chinese_attachment_labels() {
    App::test((), |mut app| async move {
        app.update(|ctx| {
            ctx.add_singleton_model(|_| Appearance::mock());
            let line =
                render_lines(snapshot("screenshot.png", 1, 1), 80, LocaleId::ZhCn, ctx).remove(0);
            assert!(line.contains("[图像]"));
            assert!(line.contains("screenshot.png"));
        });
    });
}

#[test]
fn attachment_binding_descriptions_are_localized() {
    assert_eq!(
        attachment_binding_description_for_locale(LocaleId::ZhCn, "tui.attachments.binding.next"),
        "选择下一个附件"
    );
}
