#import <AppKit/AppKit.h>
#import <objc/message.h>
#import <objc/runtime.h>

typedef void (*K2OTrackpadCallback)(double x, double y, int phase, unsigned long touchCount);

static K2OTrackpadCallback K2OCallback = NULL;

static void K2OCallSuper(id receiver, SEL selector, NSEvent *event) {
  struct objc_super superInfo = {
    .receiver = receiver,
    .super_class = class_getSuperclass(object_getClass(receiver)),
  };
  ((void (*)(struct objc_super *, SEL, id))objc_msgSendSuper)(&superInfo, selector, event);
}

static void K2OEmitTouches(id receiver, NSEvent *event, int phase) {
  if (K2OCallback == NULL) return;

  NSSet<NSTouch *> *touches = [event touchesMatchingPhase:NSTouchPhaseAny inView:receiver];
  if (touches.count == 0) return;

  NSTouch *touch = touches.anyObject;
  NSPoint position = touch.normalizedPosition;
  K2OCallback(position.x, 1.0 - position.y, phase, touches.count);
}

static void K2OTouchesBegan(id receiver, SEL selector, NSEvent *event) {
  K2OCallSuper(receiver, selector, event);
  K2OEmitTouches(receiver, event, 0);
}

static void K2OTouchesMoved(id receiver, SEL selector, NSEvent *event) {
  K2OCallSuper(receiver, selector, event);
  K2OEmitTouches(receiver, event, 1);
}

static void K2OTouchesEnded(id receiver, SEL selector, NSEvent *event) {
  K2OCallSuper(receiver, selector, event);
  K2OEmitTouches(receiver, event, 2);
}

static void K2OTouchesCancelled(id receiver, SEL selector, NSEvent *event) {
  K2OCallSuper(receiver, selector, event);
  K2OEmitTouches(receiver, event, 3);
}

void k2o_install_trackpad_capture(void *viewPointer, K2OTrackpadCallback callback) {
  if (viewPointer == NULL) return;
  K2OCallback = callback;

  NSView *view = (__bridge NSView *)viewPointer;
  Class originalClass = object_getClass(view);
  Class captureClass = NSClassFromString(@"K2OTrackpadWebView");
  if (captureClass == Nil) {
    captureClass = objc_allocateClassPair(originalClass, "K2OTrackpadWebView", 0);
    class_addMethod(captureClass, @selector(touchesBeganWithEvent:), (IMP)K2OTouchesBegan, "v@:@");
    class_addMethod(captureClass, @selector(touchesMovedWithEvent:), (IMP)K2OTouchesMoved, "v@:@");
    class_addMethod(captureClass, @selector(touchesEndedWithEvent:), (IMP)K2OTouchesEnded, "v@:@");
    class_addMethod(captureClass, @selector(touchesCancelledWithEvent:), (IMP)K2OTouchesCancelled, "v@:@");
    objc_registerClassPair(captureClass);
  }

  object_setClass(view, captureClass);
  view.allowedTouchTypes = NSTouchTypeMaskIndirect;
  view.wantsRestingTouches = YES;
}
