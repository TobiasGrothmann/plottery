// Forwards pinch, wheel, drag and click on the preview stage to Rust.
const stage = document.getElementById("preview-stage");
if (stage) {
  let gestureScale = null;
  let drag = null;

  const send = (kind, e, extra) => {
    const rect = stage.getBoundingClientRect();
    const paper = stage.querySelector(".paper");
    dioxus.send(Object.assign({
      kind,
      x: e.clientX - rect.left - rect.width / 2,
      y: e.clientY - rect.top - rect.height / 2,
      paper_w: paper ? paper.offsetWidth : 0,
      paper_h: paper ? paper.offsetHeight : 0,
      stage_w: rect.width,
      stage_h: rect.height,
      factor: 1,
      dx: 0,
      dy: 0,
    }, extra));
  };
  const onControls = (e) => e.target.closest && e.target.closest(".zoom-ctl");

  stage.addEventListener("wheel", (e) => {
    e.preventDefault();
    const unit = e.deltaMode === 1 ? 16 : 1;
    if (e.ctrlKey) {
      if (gestureScale === null) send("zoom", e, { factor: Math.exp(-e.deltaY * unit * 0.01) });
    } else {
      send("pan", e, { dx: -e.deltaX * unit, dy: -e.deltaY * unit });
    }
  }, { passive: false });

  // WebKit reports trackpad pinch as gesture events instead of ctrl+wheel.
  stage.addEventListener("gesturestart", (e) => { e.preventDefault(); gestureScale = 1; });
  stage.addEventListener("gesturechange", (e) => {
    e.preventDefault();
    send("zoom", e, { factor: e.scale / gestureScale });
    gestureScale = e.scale;
  });
  stage.addEventListener("gestureend", (e) => { e.preventDefault(); gestureScale = null; });

  stage.addEventListener("pointerdown", (e) => {
    if (e.button !== 0 || onControls(e)) return;
    drag = { x: e.clientX, y: e.clientY, moved: false };
    stage.setPointerCapture(e.pointerId);
  });
  stage.addEventListener("pointermove", (e) => {
    if (!drag) return;
    const dx = e.clientX - drag.x;
    const dy = e.clientY - drag.y;
    if (!drag.moved && Math.hypot(dx, dy) < 4) return;
    if (!drag.moved) stage.classList.add("dragging");
    drag = { x: e.clientX, y: e.clientY, moved: true };
    send("pan", e, { dx, dy });
  });
  const endDrag = (e, click) => {
    if (!drag) return;
    if (click && !drag.moved) send("click", e);
    drag = null;
    stage.classList.remove("dragging");
  };
  stage.addEventListener("pointerup", (e) => endDrag(e, true));
  stage.addEventListener("pointercancel", (e) => endDrag(e, false));
}
