import AppKit

@_silgen_name("mac_now_playing_send_command")
func mac_now_playing_send_command(_ cmd: Int32) -> Bool

@objc public class TLyricsTouchBarController: NSObject, NSTouchBarDelegate {
    public static let shared = TLyricsTouchBarController()

    private var touchBar: NSTouchBar?
    private var currentLyricLabel: NSTextField?
    private var nextLyricLabel: NSTextField?
    private var playPauseButton: NSButton?

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

            // Attach to application
            NSApplication.shared.touchBar = bar
            for window in NSApplication.shared.windows {
                window.touchBar = bar
            }
        }
    }

    public func update(current: String, next: String, isPlaying: Bool) {
        DispatchQueue.main.async { [weak self] in
            guard let self = self else { return }

            if let lbl = self.currentLyricLabel {
                lbl.stringValue = current.isEmpty ? "♪ TLyrics" : current
            }
            if let nextLbl = self.nextLyricLabel {
                nextLbl.stringValue = next.isEmpty ? "" : "‣ \(next)"
                nextLbl.isHidden = next.isEmpty
            }
            if let btn = self.playPauseButton {
                let iconName = isPlaying ? NSImage.touchBarPauseTemplateName : NSImage.touchBarPlayTemplateName
                btn.image = NSImage(named: iconName)
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
