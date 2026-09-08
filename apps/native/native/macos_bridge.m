#import <AppKit/AppKit.h>
#import <ColorSync/ColorSync.h>
#import <CoreGraphics/CoreGraphics.h>
#import <IOKit/hidsystem/IOLLEvent.h>
#import <objc/message.h>
#import <objc/runtime.h>

typedef struct {
  double x;
  double y;
  double width;
  double height;
} K2ORelativeFrame;

typedef bool (*K2OTrackpadCallback)(double x, double y, int phase,
                                    unsigned long touchCount);
typedef void (*K2ODismissCallback)(void);

static K2OTrackpadCallback K2OTrackpadHandler = NULL;
static K2ODismissCallback K2ODismissHandler = NULL;
static K2ODismissCallback K2OFinishTextHandler = NULL;
static NSPoint K2OLastTouchPosition;
static NSTimeInterval K2OLastTouchTimestamp = 0.0;
static NSUInteger K2OLastTouchCount = 0;
static bool K2OTouchIsActive = false;
static bool K2OWindowDragIsActive = false;
static NSPoint K2OWindowDragLocalAnchor;

static NSScreen *K2OScreenWithID(uint64_t displayID) {
  for (NSScreen *screen in NSScreen.screens) {
    NSNumber *number = screen.deviceDescription[@"NSScreenNumber"];
    if (number.unsignedLongLongValue == displayID) return screen;
  }
  return nil;
}

static NSScreen *K2OPointerScreen(void) {
  NSPoint pointer = NSEvent.mouseLocation;
  for (NSScreen *screen in NSScreen.screens) {
    if (NSPointInRect(pointer, screen.frame)) return screen;
  }
  return NSScreen.mainScreen;
}

static uint64_t K2OScreenID(NSScreen *screen) {
  NSNumber *number = screen.deviceDescription[@"NSScreenNumber"];
  uint32_t displayID = number.unsignedIntValue;
  CFUUIDRef uuid = CGDisplayCreateUUIDFromDisplayID(displayID);
  if (uuid == NULL) return displayID;

  CFUUIDBytes bytes = CFUUIDGetUUIDBytes(uuid);
  const uint8_t *raw = (const uint8_t *)&bytes;
  uint64_t hash = 1469598103934665603ULL;
  for (size_t index = 0; index < sizeof(bytes); index++) {
    hash ^= raw[index];
    hash *= 1099511628211ULL;
  }
  CFRelease(uuid);
  return hash;
}

void k2o_set_accessory_application(void) {
  [NSApp setActivationPolicy:NSApplicationActivationPolicyAccessory];
}

bool k2o_system_dark_mode(void) {
  NSAppearance *appearance = NSApp.effectiveAppearance;
  NSString *match = [appearance bestMatchFromAppearancesWithNames:@[
    NSAppearanceNameAqua, NSAppearanceNameDarkAqua
  ]];
  return [match isEqualToString:NSAppearanceNameDarkAqua];
}

bool k2o_invocation_access_granted(void) {
  return CGPreflightListenEventAccess();
}

bool k2o_request_invocation_access(void) {
  return CGRequestListenEventAccess();
}

bool k2o_open_invocation_access_settings(void) {
  NSURL *url = [NSURL URLWithString:
      @"x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"];
  return url != nil && [NSWorkspace.sharedWorkspace openURL:url];
}

uint32_t k2o_modifier_key_state(void) {
  const CGEventSourceStateID source = kCGEventSourceStateCombinedSessionState;
  uint32_t state = 0;
  if (CGEventSourceKeyState(source, 55)) state |= 1u << 0; // Left Command
  if (CGEventSourceKeyState(source, 54)) state |= 1u << 1; // Right Command
  if (CGEventSourceKeyState(source, 58)) state |= 1u << 2; // Left Option
  if (CGEventSourceKeyState(source, 61)) state |= 1u << 3; // Right Option
  if (CGEventSourceKeyState(source, 56)) state |= 1u << 4; // Left Shift
  if (CGEventSourceKeyState(source, 60)) state |= 1u << 5; // Right Shift
  return state;
}

uint32_t k2o_modifier_mask_from_flags(uint64_t flags) {
  uint32_t state = 0;
  if (flags & NX_DEVICELCMDKEYMASK) state |= 1u << 0;
  if (flags & NX_DEVICERCMDKEYMASK) state |= 1u << 1;
  if (flags & NX_DEVICELALTKEYMASK) state |= 1u << 2;
  if (flags & NX_DEVICERALTKEYMASK) state |= 1u << 3;
  if (flags & NX_DEVICELSHIFTKEYMASK) state |= 1u << 4;
  if (flags & NX_DEVICERSHIFTKEYMASK) state |= 1u << 5;
  return state;
}

typedef void (*K2OInvocationCallback)(void *, uint32_t, uint16_t, bool);
typedef struct {
  K2OInvocationCallback callback;
  void *context;
  CFMachPortRef tap;
} K2OInvocationListener;

static CGEventRef K2OInvocationEvent(CGEventTapProxy proxy, CGEventType type,
                                   CGEventRef event, void *context) {
  (void)proxy;
  K2OInvocationListener *listener = context;
  if (type == kCGEventTapDisabledByTimeout ||
      type == kCGEventTapDisabledByUserInput) {
    if (CGPreflightListenEventAccess()) {
      CGEventTapEnable(listener->tap, true);
    } else {
      CFRunLoopStop(CFRunLoopGetCurrent());
    }
    return event;
  }
  if (event == NULL) return event;
  uint16_t key = (uint16_t)CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode);
  // Invocation only needs modifier transitions and Space, never typed text.
  if (type != kCGEventFlagsChanged && key != 49) return event;
  uint32_t mask = k2o_modifier_mask_from_flags(CGEventGetFlags(event));
  bool down = type == kCGEventKeyDown;
  if (type == kCGEventFlagsChanged) {
    switch (key) {
      case 55: down = (mask & (1u << 0)) != 0; break;
      case 54: down = (mask & (1u << 1)) != 0; break;
      case 58: down = (mask & (1u << 2)) != 0; break;
      case 61: down = (mask & (1u << 3)) != 0; break;
      case 56: down = (mask & (1u << 4)) != 0; break;
      case 60: down = (mask & (1u << 5)) != 0; break;
      default: return event;
    }
  }
  listener->callback(listener->context, mask, key, down);
  return event;
}

bool k2o_listen_invocation(K2OInvocationCallback callback, void *context) {
  @autoreleasepool {
    K2OInvocationListener listener = {callback, context, NULL};
    CGEventMask mask = CGEventMaskBit(kCGEventFlagsChanged) |
                       CGEventMaskBit(kCGEventKeyDown) | CGEventMaskBit(kCGEventKeyUp);
    listener.tap = CGEventTapCreate(kCGSessionEventTap, kCGHeadInsertEventTap,
                                   kCGEventTapOptionListenOnly, mask,
                                   K2OInvocationEvent, &listener);
    if (listener.tap == NULL) return false;
    CFRunLoopSourceRef source = CFMachPortCreateRunLoopSource(NULL, listener.tap, 0);
    if (source == NULL) {
      CFRelease(listener.tap);
      return false;
    }
    CFRunLoopAddSource(CFRunLoopGetCurrent(), source, kCFRunLoopCommonModes);
    CGEventTapEnable(listener.tap, true);
    CFRunLoopRun();
    CFRunLoopRemoveSource(CFRunLoopGetCurrent(), source, kCFRunLoopCommonModes);
    CFMachPortInvalidate(listener.tap);
    CFRelease(source);
    CFRelease(listener.tap);
    return false;
  }
}

int64_t k2o_frontmost_application_pid(void) {
  NSRunningApplication *application =
      NSWorkspace.sharedWorkspace.frontmostApplication;
  if (application == nil || application.processIdentifier == NSProcessInfo.processInfo.processIdentifier) {
    return 0;
  }
  return application.processIdentifier;
}

bool k2o_activate_application(int64_t pid) {
  NSRunningApplication *application =
      [NSRunningApplication runningApplicationWithProcessIdentifier:(pid_t)pid];
  if (application == nil || application.terminated) return false;
  return [application activateWithOptions:NSApplicationActivateAllWindows];
}

void k2o_configure_window(void *viewPointer) {
  if (viewPointer == NULL) return;
  NSView *view = (__bridge NSView *)viewPointer;
  NSWindow *window = view.window;
  if (window == nil) return;

  window.styleMask = NSWindowStyleMaskBorderless | NSWindowStyleMaskResizable |
                     NSWindowStyleMaskFullSizeContentView;
  window.level = NSNormalWindowLevel;
  window.collectionBehavior = NSWindowCollectionBehaviorCanJoinAllSpaces |
                              NSWindowCollectionBehaviorFullScreenAuxiliary;
  window.hidesOnDeactivate = NO;
  window.hasShadow = YES;
  window.movable = YES;
  window.opaque = NO;
  window.backgroundColor = NSColor.clearColor;
  window.minSize = NSMakeSize(680.0, 420.0);
  window.releasedWhenClosed = NO;

  view.wantsLayer = YES;
  view.layer.cornerRadius = 8.0;
  view.layer.masksToBounds = YES;
}

void k2o_show_window(void *viewPointer) {
  if (viewPointer == NULL) return;
  NSView *view = (__bridge NSView *)viewPointer;
  NSWindow *window = view.window;
  if (window == nil) return;
  [NSApp activateIgnoringOtherApps:YES];
  [window makeKeyAndOrderFront:nil];
  // A borderless window can become key before its input view is the responder.
  // Establish it when showing the canvas, without disturbing an ongoing IME.
  if (window.firstResponder != view) [window makeFirstResponder:view];
}

void k2o_request_window_drag(void *viewPointer, double x, double y) {
  if (viewPointer == NULL) return;
  NSWindow *window = ((__bridge NSView *)viewPointer).window;
  if (window != nil) {
    K2OWindowDragIsActive = true;
    K2OWindowDragLocalAnchor = NSMakePoint(x, y);
  }
}

void k2o_update_window_drag(void *viewPointer, double x, double y) {
  if (!K2OWindowDragIsActive || viewPointer == NULL) return;
  NSWindow *window = ((__bridge NSView *)viewPointer).window;
  if (window == nil) return;

  NSPoint current = window.frame.origin;
  NSPoint origin = NSMakePoint(current.x + x - K2OWindowDragLocalAnchor.x,
                               current.y - y + K2OWindowDragLocalAnchor.y);
  [window setFrameOrigin:origin];
}

void k2o_end_window_drag(void *viewPointer, double x, double y) {
  k2o_update_window_drag(viewPointer, x, y);
  K2OWindowDragIsActive = false;
}

uint64_t k2o_pointer_display_id(void) {
  return K2OScreenID(K2OPointerScreen());
}

bool k2o_place_window(void *viewPointer, uint64_t displayID,
                      bool hasSavedFrame, K2ORelativeFrame relative) {
  if (viewPointer == NULL) return false;
  NSWindow *window = ((__bridge NSView *)viewPointer).window;
  NSScreen *screen = K2OScreenWithID(displayID) ?: K2OPointerScreen();
  if (window == nil || screen == nil) return false;

  NSRect work = screen.visibleFrame;
  NSSize size = window.frame.size;
  NSPoint origin;
  if (hasSavedFrame) {
    origin.x = work.origin.x + relative.x * work.size.width;
    origin.y = work.origin.y + relative.y * work.size.height;
    double maxX = fmax(NSMinX(work), NSMaxX(work) - size.width);
    double maxY = fmax(NSMinY(work), NSMaxY(work) - size.height);
    origin.x = fmin(fmax(origin.x, NSMinX(work)), maxX);
    origin.y = fmin(fmax(origin.y, NSMinY(work)), maxY);
  } else {
    origin.x = NSMidX(work) - size.width * 0.5;
    origin.y = NSMidY(work) - size.height * 0.5;
  }
  [window setFrameOrigin:origin];
  return true;
}

bool k2o_preferred_window_size(void *viewPointer, bool hasSavedFrame,
                               K2ORelativeFrame relative, double *width,
                               double *height) {
  if (viewPointer == NULL || width == NULL || height == NULL) return false;
  NSWindow *window = ((__bridge NSView *)viewPointer).window;
  NSScreen *screen = window.screen ?: K2OPointerScreen();
  if (screen == nil) return false;

  NSRect work = screen.visibleFrame;
  double widthRatio = hasSavedFrame ? relative.width : 0.5;
  double heightRatio = hasSavedFrame ? relative.height : 0.5;
  widthRatio = fmin(fmax(widthRatio, 0.15), 1.0);
  heightRatio = fmin(fmax(heightRatio, 0.15), 1.0);
  *width = fmin(work.size.width, fmax(680.0, work.size.width * widthRatio));
  *height = fmin(work.size.height, fmax(420.0, work.size.height * heightRatio));
  return *width > 0.0 && *height > 0.0;
}

bool k2o_current_relative_frame(void *viewPointer, uint64_t *displayID,
                                K2ORelativeFrame *relative) {
  if (viewPointer == NULL || displayID == NULL || relative == NULL) return false;
  NSWindow *window = ((__bridge NSView *)viewPointer).window;
  NSScreen *screen = window.screen;
  if (window == nil || screen == nil) return false;

  NSRect frame = window.frame;
  NSRect work = screen.visibleFrame;
  if (work.size.width <= 0.0 || work.size.height <= 0.0) return false;
  *displayID = K2OScreenID(screen);
  relative->x = (frame.origin.x - work.origin.x) / work.size.width;
  relative->y = (frame.origin.y - work.origin.y) / work.size.height;
  relative->width = frame.size.width / work.size.width;
  relative->height = frame.size.height / work.size.height;
  return true;
}

static void K2OCallSuper(id receiver, SEL selector, NSEvent *event) {
  struct objc_super superInfo = {
      .receiver = receiver,
      .super_class = class_getSuperclass(object_getClass(receiver)),
  };
  ((void (*)(struct objc_super *, SEL, id))objc_msgSendSuper)(&superInfo,
                                                               selector, event);
}

static void K2OEmitTouches(id receiver, NSEvent *event, int phase) {
  if (K2OTrackpadHandler == NULL) return;
  NSSet<NSTouch *> *touches =
      [event touchesMatchingPhase:NSTouchPhaseAny inView:receiver];
  if (touches.count == 0) {
    if (phase == 2 || phase == 3) {
      K2OTouchIsActive = false;
      K2OLastTouchCount = 0;
    }
    return;
  }
  NSTouch *touch = touches.anyObject;
  NSPoint position = touch.normalizedPosition;
  K2OLastTouchPosition = NSMakePoint(position.x, 1.0 - position.y);
  K2OLastTouchTimestamp = event.timestamp;
  K2OLastTouchCount = touches.count;
  K2OTouchIsActive = (phase == 0 || phase == 1) && touches.count == 1;
  K2OTrackpadHandler(position.x, 1.0 - position.y, phase, touches.count);
}

static void K2OMouseDown(id receiver, SEL selector, NSEvent *event) {
  if (K2OTrackpadHandler == NULL) {
    K2OCallSuper(receiver, selector, event);
    return;
  }
  NSTimeInterval elapsed = event.timestamp - K2OLastTouchTimestamp;
  bool recentlyEnded = K2OLastTouchCount == 1 && elapsed >= 0.0 && elapsed <= 0.45;
  if (!K2OTouchIsActive && !recentlyEnded) {
    K2OCallSuper(receiver, selector, event);
    return;
  }
  bool consumed = K2OTrackpadHandler(K2OLastTouchPosition.x,
                                     K2OLastTouchPosition.y, 4, 1);
  if (!K2OTouchIsActive) K2OLastTouchCount = 0;
  if (!consumed) K2OCallSuper(receiver, selector, event);
}

static void K2OKeyDown(id receiver, SEL selector, NSEvent *event) {
  bool commandReturn = (event.modifierFlags & NSEventModifierFlagCommand) != 0 &&
                       (event.keyCode == 36 || event.keyCode == 76);
  if (commandReturn && K2OFinishTextHandler != NULL) {
    // Leave active composition with the input method; never discard preedit.
    if ([receiver respondsToSelector:@selector(hasMarkedText)] &&
        [receiver hasMarkedText]) {
      return;
    } else if (!event.isARepeat) {
      K2OFinishTextHandler();
    }
    return;
  }
  NSString *characters = event.charactersIgnoringModifiers.lowercaseString;
  bool commandW = (event.modifierFlags & NSEventModifierFlagCommand) != 0 &&
                  [characters isEqualToString:@"w"];
  if (commandW && K2ODismissHandler != NULL) {
    K2ODismissHandler();
    return;
  }
  K2OCallSuper(receiver, selector, event);
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

void k2o_install_trackpad_capture(void *viewPointer,
                                  K2OTrackpadCallback callback,
                                  K2ODismissCallback dismissCallback,
                                  K2ODismissCallback finishTextCallback) {
  if (viewPointer == NULL) return;
  K2OTrackpadHandler = callback;
  K2ODismissHandler = dismissCallback;
  K2OFinishTextHandler = finishTextCallback;

  NSView *view = (__bridge NSView *)viewPointer;
  Class originalClass = object_getClass(view);
  Class captureClass = NSClassFromString(@"K2OTrackpadNativeView");
  if (captureClass == Nil) {
    captureClass = objc_allocateClassPair(originalClass, "K2OTrackpadNativeView", 0);
    class_addMethod(captureClass, @selector(touchesBeganWithEvent:),
                    (IMP)K2OTouchesBegan, "v@:@");
    class_addMethod(captureClass, @selector(touchesMovedWithEvent:),
                    (IMP)K2OTouchesMoved, "v@:@");
    class_addMethod(captureClass, @selector(touchesEndedWithEvent:),
                    (IMP)K2OTouchesEnded, "v@:@");
    class_addMethod(captureClass, @selector(touchesCancelledWithEvent:),
                    (IMP)K2OTouchesCancelled, "v@:@");
    class_addMethod(captureClass, @selector(mouseDown:),
                    (IMP)K2OMouseDown, "v@:@");
    class_addMethod(captureClass, @selector(keyDown:),
                    (IMP)K2OKeyDown, "v@:@");
    objc_registerClassPair(captureClass);
  }

  if (![view isKindOfClass:captureClass]) object_setClass(view, captureClass);
  view.allowedTouchTypes = NSTouchTypeMaskIndirect;
  view.wantsRestingTouches = YES;
}
