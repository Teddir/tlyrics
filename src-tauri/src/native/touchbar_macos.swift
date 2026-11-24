import AppKit

@_silgen_name("mac_now_playing_send_command")
func mac_now_playing_send_command(_ cmd: Int32) -> Bool

// DFRFoundation private functions
typealias DFRElementSetControlStripPresenceFunction = @convention(c) (NSTouchBarItem.Identifier, Bool) -> Void
typealias DFRSystemModalShowsCloseBoxFunction = @convention(c) (Bool) -> Void

private let dfrBundle: CFBundle? = {
    CFBundleCreate(kCFAllocatorDefault, NSURL(fileURLWithPath: "/System/Library/PrivateFrameworks/DFRFoundation.framework"))
}()

private let setControlStripPresence: DFRElementSetControlStripPresenceFunction? = {
    guard let bundle = dfrBundle,
          let ptr = CFBundleGetFunctionPointerForName(bundle, "DFRElementSetControlStripPresenceForIdentifier" as CFString) else {
        return nil
    }
    return unsafeBitCast(ptr, to: DFRElementSetControlStripPresenceFunction.self)
}()

private let setShowsCloseBox: DFRSystemModalShowsCloseBoxFunction? = {
    guard let bundle = dfrBundle,
          let ptr = CFBundleGetFunctionPointerForName(bundle, "DFRSystemModalShowsCloseBoxWhenFrontMost" as CFString) else {
        return nil
    }
    return unsafeBitCast(ptr, to: DFRSystemModalShowsCloseBoxFunction.self)
}()

@objc public class TLyricsTouchBarController: NSObject, NSTouchBarDelegate {
    public static let shared = TLyricsTouchBarController()

    private var touchBar: NSTouchBar?
    private var controlStripItem: NSCustomTouchBarItem?
    private var currentLyricLabel: NSTextField?
    private var nextLyricLabel: NSTextField?
    private var playPauseButton: NSButton?
    private var isSystemModalPresented = false

    private let trayItemIdentifier = NSTouchBarItem.Identifier("com.mac.tlyrics.tray")
    private let prevItemIdentifier = NSTouchBarItem.Identifier("com.mac.tlyrics.prev")
    private let playPauseItemIdentifier = NSTouchBarItem.Identifier("com.mac.tlyrics.playpause")
    private let nextItemIdentifier = NSTouchBarItem.Identifier("com.mac.tlyrics.next")
    private let currentLyricIdentifier = NSTouchBarItem.Identifier("com.mac.tlyrics.currentLyric")
    private let nextLyricIdentifier = NSTouchBarItem.Identifier("com.mac.tlyrics.nextLyric")

    public override init() {
        super.init()
    }

    public func setupTouchBar() {
        DispatchQueue.main.async { [weak self] in
            guard let self = self else { return }

            // 1. Build the Main NSTouchBar
            let bar = NSTouchBar()
            bar.delegate = self
            bar.customizationIdentifier = NSTouchBar.CustomizationIdentifier("com.mac.tlyrics.touchbar")
            bar.defaultItemIdentifiers = [
                self.prevItemIdentifier,
                self.playPauseItemIdentifier,
                self.nextItemIdentifier,
                NSTouchBarItem.Identifier.fixedSpaceSmall,
                self.currentLyricIdentifier,
                self.nextLyricIdentifier,
                NSTouchBarItem.Identifier.flexibleSpace
            ]
            self.touchBar = bar

            // Attach to application and windows (for foreground focus)
            NSApplication.shared.touchBar = bar
            for window in NSApplication.shared.windows {
                window.touchBar = bar
            }

            // 2. Setup System Tray item in the Control Strip (for background persistence)
            self.setupControlStripItem(with: bar)

            // 3. Immediately present globally as a system modal Touch Bar
            self.presentGlobalTouchBar()
        }
    }

    private func setupControlStripItem(with bar: NSTouchBar) {
        let tray = NSCustomTouchBarItem(identifier: self.trayItemIdentifier)
        let btn = NSButton(
            title: "♪",
            target: self,
            action: #selector(self.onTrayIconTapped)
        )
        btn.bezelColor = NSColor.systemTeal
        tray.view = btn
        self.controlStripItem = tray

        // Register with system tray via private selector
        let addSelector = NSSelectorFromString("addSystemTrayItem:")
        if (NSTouchBarItem.self as AnyObject).responds(to: addSelector) {
            _ = (NSTouchBarItem.self as AnyObject).perform(addSelector, with: tray)
        }

        // Enable Control Strip presence
        setControlStripPresence?(self.trayItemIdentifier, true)
        setShowsCloseBox?(false)
    }

    @objc public func onTrayIconTapped() {
        if self.isSystemModalPresented {
            self.dismissGlobalTouchBar()
        } else {
            self.presentGlobalTouchBar()
        }
    }

    public func presentGlobalTouchBar() {
        guard let bar = self.touchBar else { return }

        DispatchQueue.main.async { [weak self] in
            guard let self = self else { return }

            // Present System Modal Touch Bar over all applications
            let presentSelector = NSSelectorFromString("presentSystemModalTouchBar:systemTrayItemIdentifier:")
            if (NSTouchBar.self as AnyObject).responds(to: presentSelector) {
                _ = (NSTouchBar.self as AnyObject).perform(presentSelector, with: bar, with: self.trayItemIdentifier)
                self.isSystemModalPresented = true
            }
        }
    }

    public func dismissGlobalTouchBar() {
        guard let bar = self.touchBar else { return }

        DispatchQueue.main.async { [weak self] in
            guard let self = self else { return }

            let dismissSelector = NSSelectorFromString("dismissSystemModalTouchBar:")
            if (NSTouchBar.self as AnyObject).responds(to: dismissSelector) {
                _ = (NSTouchBar.self as AnyObject).perform(dismissSelector, with: bar)
                self.isSystemModalPresented = false
            }
        }
    }

    public func update(current: String, next: String, isPlaying: Bool) {
        DispatchQueue.main.async { [weak self] in
            guard let self = self else { return }

            let displayCurrent = current.isEmpty ? "♪ TLyrics" : current
            let displayNext = next.isEmpty ? "" : "‣ \(next)"

            if let lbl = self.currentLyricLabel {
                lbl.stringValue = displayCurrent
            }
            if let nextLbl = self.nextLyricLabel {
                nextLbl.stringValue = displayNext
                nextLbl.isHidden = next.isEmpty
            }
            if let btn = self.playPauseButton {
                let iconName = isPlaying ? NSImage.touchBarPauseTemplateName : NSImage.touchBarPlayTemplateName
                btn.image = NSImage(named: iconName)
            }

            // Also update the mini Control Strip button title if music is playing
            if let trayBtn = self.controlStripItem?.view as? NSButton {
                trayBtn.title = isPlaying ? "♪" : "❚❚"
            }

            // Ensure the system modal Touch Bar remains presented when playback is active
            if isPlaying && !self.isSystemModalPresented {
                self.presentGlobalTouchBar()
            }
        }
    }

    public func touchBar(_ touchBar: NSTouchBar, makeItemForIdentifier identifier: NSTouchBarItem.Identifier) -> NSTouchBarItem? {
        switch identifier {
        case prevItemIdentifier:
            let item = NSCustomTouchBarItem(identifier: identifier)
            let btn = NSButton(
                image: NSImage(named: NSImage.touchBarSkipBackTemplateName)!,
                target: self,
                action: #selector(onPrevClicked)
            )
            item.view = btn
            return item

        case playPauseItemIdentifier:
            let item = NSCustomTouchBarItem(identifier: identifier)
            let btn = NSButton(
                image: NSImage(named: NSImage.touchBarPlayTemplateName)!,
                target: self,
                action: #selector(onPlayPauseClicked)
            )
            self.playPauseButton = btn
            item.view = btn
            return item

        case nextItemIdentifier:
            let item = NSCustomTouchBarItem(identifier: identifier)
            let btn = NSButton(
                image: NSImage(named: NSImage.touchBarSkipAheadTemplateName)!,
                target: self,
                action: #selector(onNextClicked)
            )
            item.view = btn
            return item

        case currentLyricIdentifier:
            let item = NSCustomTouchBarItem(identifier: identifier)
            let label = NSTextField(labelWithString: "♪ TLyrics")
            label.font = NSFont.systemFont(ofSize: 14, weight: .bold)
            label.textColor = NSColor.white
            label.lineBreakMode = .byTruncatingTail
            label.maximumNumberOfLines = 1
            label.setContentHuggingPriority(.defaultLow, for: .horizontal)
            label.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
            self.currentLyricLabel = label
            item.view = label
            return item

        case nextLyricIdentifier:
            let item = NSCustomTouchBarItem(identifier: identifier)
            let label = NSTextField(labelWithString: "")
            label.font = NSFont.systemFont(ofSize: 12, weight: .regular)
            label.textColor = NSColor.lightGray
            label.lineBreakMode = .byTruncatingTail
            label.maximumNumberOfLines = 1
            label.setContentHuggingPriority(.defaultLow, for: .horizontal)
            label.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
            self.nextLyricLabel = label
            item.view = label
            return item

        default:
            return nil
        }
    }

    @objc private func onPrevClicked() {
        _ = mac_now_playing_send_command(5) // PreviousTrack
    }

    @objc private func onPlayPauseClicked() {
        _ = mac_now_playing_send_command(2) // TogglePlayPause
    }

    @objc private func onNextClicked() {
        _ = mac_now_playing_send_command(4) // NextTrack
    }
}

// C-Bridge exports
@_cdecl("tlyrics_touchbar_init")
public func tlyricsTouchBarInit() {
    TLyricsTouchBarController.shared.setupTouchBar()
}

@_cdecl("tlyrics_touchbar_present_global")
public func tlyricsTouchBarPresentGlobal() {
    TLyricsTouchBarController.shared.presentGlobalTouchBar()
}

@_cdecl("tlyrics_touchbar_dismiss_global")
public func tlyricsTouchBarDismissGlobal() {
    TLyricsTouchBarController.shared.dismissGlobalTouchBar()
}

@_cdecl("tlyrics_touchbar_update")
public func tlyricsTouchBarUpdate(
    current: UnsafePointer<CChar>?,
    next: UnsafePointer<CChar>?,
    isPlaying: Bool
) {
    let currStr = current != nil ? String(cString: current!) : ""
    let nextStr = next != nil ? String(cString: next!) : ""
    TLyricsTouchBarController.shared.update(current: currStr, next: nextStr, isPlaying: isPlaying)
}
