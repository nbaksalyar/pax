//
//  ContentView.swift
//  pax-dev-harness-macos
//
//  Created by Zachary Brown on 4/6/22.
//

import SwiftUI
import Foundation

class TextElements: ObservableObject {
    static let singleton : TextElements = TextElements()

    @Published var elements : [[UInt64]: TextElement] = [:]

    func add(element: TextElement) {
        self.elements[element.id_chain] = element
    }
    func remove(id: [UInt64]) {
        self.elements.removeValue(forKey: id)
    }
}

class FrameElements: ObservableObject {
    static let singleton : FrameElements = FrameElements()

    @Published var elements : [[UInt64]: FrameElement] = [:]

    func add(element: FrameElement) {
        self.elements[element.id_chain] = element
    }
    func remove(id: [UInt64]) {
        self.elements.removeValue(forKey: id)
    }
    func get(id: [UInt64]) -> FrameElement? {
        return self.elements[id]
    }
}

struct PaxView: View {

    var canvasView : some View = PaxCanvasViewRepresentable()
            .frame(minWidth: 300, maxWidth: .infinity, minHeight: 300, maxHeight: .infinity)

    var body: some View {
        ZStack {
            self.canvasView
            NativeRenderingLayer()
        }
        .onAppear {
            registerFonts()
        }.gesture(DragGesture(minimumDistance: 0, coordinateSpace: .global).onEnded { dragGesture in
                    //FUTURE: especially if parsing is a bottleneck, could use a different encoding than JSON
            let json = String(format: "{\"Click\": {\"x\": %f, \"y\": %f, \"button\": \"Left\", \"modifiers\":[] } }", dragGesture.location.x, dragGesture.location.y);
                    let buffer = try! FlexBufferBuilder.fromJSON(json)

                    //Send `Click` interrupt
                    buffer.data.withUnsafeBytes({ptr in
                        var ffi_container = InterruptBuffer( data_ptr: ptr.baseAddress!, length: UInt64(ptr.count) )

                        withUnsafePointer(to: &ffi_container) {ffi_container_ptr in
                            pax_interrupt(PaxEngineContainer.paxEngineContainer!, ffi_container_ptr)
                        }
                    })
        })

    }

    func registerFonts() {
        guard let resourceURL = Bundle.main.resourceURL else { return }
        let fontFileExtensions = ["ttf", "otf"]

        do {
            let resourceFiles = try FileManager.default.contentsOfDirectory(at: resourceURL, includingPropertiesForKeys: nil, options: [])
            for fileURL in resourceFiles {
                let fileExtension = fileURL.pathExtension.lowercased()
                if fontFileExtensions.contains(fileExtension) {
                    let fontDescriptors = CTFontManagerCreateFontDescriptorsFromURL(fileURL as CFURL) as! [CTFontDescriptor]
                    if let fontDescriptor = fontDescriptors.first,
                       let postscriptName = CTFontDescriptorCopyAttribute(fontDescriptor, kCTFontNameAttribute) as? String,
                       let fontFamily = CTFontDescriptorCopyAttribute(fontDescriptor, kCTFontFamilyNameAttribute) as? String {
                        if !PaxFont.isFontRegistered(fontFamily: postscriptName) {
                            var errorRef: Unmanaged<CFError>?
                            if !CTFontManagerRegisterFontsForURL(fileURL as CFURL, .process, &errorRef) {
                                print("Error registering font: \(fontFamily) - PostScript name: \(postscriptName) - \(String(describing: errorRef))")
                            }
                        } else {
                            print("Font already registered: \(fontFamily) - PostScript name: \(postscriptName)")
                        }
                    }
                }
            }
        } catch {
            print("Error reading font files from resources: \(error)")
        }
    }




    class PaxEngineContainer {
        static var paxEngineContainer : OpaquePointer? = nil
    }

    struct NativeRenderingLayer: View {

        @ObservedObject var textElements : TextElements = TextElements.singleton
        @ObservedObject var frameElements : FrameElements = FrameElements.singleton

        func getClippingMask(clippingIds: [[UInt64]]) -> some View {

            var elements : [FrameElement] = []

            clippingIds.makeIterator().forEach( { id_chain in
                elements.insert(self.frameElements.elements[id_chain]!, at: 0)
            })

            return ZStack { ForEach(elements, id: \.id_chain) { frameElement in
                Rectangle()
                        .frame(width: CGFloat(frameElement.size_x), height: CGFloat(frameElement.size_y))
                        .position(x: CGFloat(frameElement.size_x / 2.0), y: CGFloat(frameElement.size_y / 2.0))
                        .transformEffect(CGAffineTransform.init(
                                a: CGFloat(frameElement.transform[0]),
                                b: CGFloat(frameElement.transform[1]),
                                c: CGFloat(frameElement.transform[2]),
                                d: CGFloat(frameElement.transform[3]),
                                tx: CGFloat(frameElement.transform[4]),
                                ty: CGFloat(frameElement.transform[5]))
                        )
            } }
        }

        @ViewBuilder
        func getPositionedTextGroup(textElement: TextElement) -> some View {
            let transform = CGAffineTransform.init(
                a: CGFloat(textElement.transform[0]),
                b: CGFloat(textElement.transform[1]),
                c: CGFloat(textElement.transform[2]),
                d: CGFloat(textElement.transform[3]),
                tx: CGFloat(textElement.transform[4]),
                ty: CGFloat(textElement.transform[5])
            )
            var text: AttributedString {
                var attributedString: AttributedString = try! AttributedString(markdown: textElement.content, options: AttributedString.MarkdownParsingOptions(interpretedSyntax: .inlineOnlyPreservingWhitespace))

                for run in attributedString.runs {
                    if run.link != nil {
                        if let linkStyle = textElement.style_link {
                            attributedString[run.range].font = linkStyle.font.getFont(size: linkStyle.font_size)
                            if(linkStyle.underline){
                                attributedString[run.range].underlineStyle = .single
                            } else {
                                attributedString[run.range].underlineStyle = .none
                            }
                            attributedString[run.range].foregroundColor = linkStyle.fill
                        }
                    }
                }
                return attributedString

            }
            let textView : some View =
                Text(text)
                .foregroundColor(textElement.textStyle.fill)
                .font(textElement.textStyle.font.getFont(size: textElement.textStyle.font_size))
                .frame(width: CGFloat(textElement.size_x), height: CGFloat(textElement.size_y), alignment: textElement.textStyle.alignment)
                .position(x: CGFloat(textElement.size_x / 2.0), y: CGFloat(textElement.size_y / 2.0))
                .transformEffect(transform)
                .textSelection(.enabled)

//
//            if !textElement.clipping_ids.isEmpty {
//                textView.mask(getClippingMask(clippingIds: textElement.clipping_ids))
//            } else {
//                textView
//            }
            textView
        }

        var body: some View {
            ZStack{
               ForEach(Array(self.textElements.elements.values), id: \.id_chain) { textElement in
                    getPositionedTextGroup(textElement: textElement)
                }
            }

        }
    }

    struct PaxCanvasViewRepresentable: NSViewRepresentable {
        typealias NSViewType = PaxCanvasView

        func makeNSView(context: Context) -> PaxCanvasView {
            let view = PaxCanvasView()
            return view
        }

        func updateNSView(_ canvas: PaxCanvasView, context: Context) { }
    }


    class PaxCanvasView: NSView {

        @ObservedObject var textElements = TextElements.singleton
        @ObservedObject var frameElements = FrameElements.singleton

        private var displayLink: CVDisplayLink?

        override init(frame frameRect: NSRect) {
            super.init(frame: frameRect)
            self.wantsLayer = true
            self.layer?.drawsAsynchronously = true
            createDisplayLink()
        }

        required init?(coder: NSCoder) {
            super.init(coder: coder)
            createDisplayLink()
        }

        private var requestAnimationFrameQueue: [() -> Void] = []

        private func processRequestAnimationFrameQueue() {
            // Execute and remove each closure in the array
            while !requestAnimationFrameQueue.isEmpty {
                let closure = requestAnimationFrameQueue.removeFirst()
                closure()
            }
        }

        func requestAnimationFrame(_ closure: @escaping () -> Void) {
            requestAnimationFrameQueue.append(closure)
        }

        private func createDisplayLink() {
            CVDisplayLinkCreateWithActiveCGDisplays(&displayLink)
            CVDisplayLinkSetOutputHandler(displayLink!) { [weak self] (_, _, _, _, _) -> CVReturn in
                DispatchQueue.main.async {
                    self?.setNeedsDisplay(self?.bounds ?? NSRect.zero)
                    self?.processRequestAnimationFrameQueue()
                }
                return kCVReturnSuccess
            }
            CVDisplayLinkStart(displayLink!)
        }

        deinit {
            CVDisplayLinkStop(displayLink!)
        }

        override func draw(_ dirtyRect: NSRect) {

            super.draw(dirtyRect)
            guard let context = NSGraphicsContext.current else { return }
            var cgContext = context.cgContext

            if PaxEngineContainer.paxEngineContainer == nil {
                let swiftLoggerCallback : @convention(c) (UnsafePointer<CChar>?) -> () = {
                    (msg) -> () in
                    let outputString = String(cString: msg!)
                    print(outputString)
                }

                //For manual debugger attachment:
//                do {
//                    sleep(10)
//                }

                PaxEngineContainer.paxEngineContainer = pax_init(swiftLoggerCallback)
            } else {

                let nativeMessageQueue = pax_tick(PaxEngineContainer.paxEngineContainer!, &cgContext, CFloat(dirtyRect.width), CFloat(dirtyRect.height))
                processNativeMessageQueue(queue: nativeMessageQueue.unsafelyUnwrapped.pointee)
                pax_dealloc_message_queue(nativeMessageQueue)
            }

            //This DispatchWorkItem `cancel()` is required because sometimes `draw` will be triggered externally from this loop, which
            //would otherwise create new families of continuously reproducing DispatchWorkItems, each ticking up a frenzy, well past the bounds of our target FPS.
            //This cancellation + shared singleton (`tickWorkItem`) ensures that only one DispatchWorkItem is enqueued at a time.
            if currentTickWorkItem != nil {
                currentTickWorkItem!.cancel()
            }

            currentTickWorkItem = DispatchWorkItem {
                self.setNeedsDisplay(dirtyRect)
                self.displayIfNeeded()
            }

        }

        var currentTickWorkItem : DispatchWorkItem? = nil

        func handleTextCreate(patch: AnyCreatePatch) {
            textElements.add(element: TextElement.makeDefault(id_chain: patch.id_chain, clipping_ids: patch.clipping_ids))
        }

        func handleTextUpdate(patch: TextUpdatePatch) {
            textElements.elements[patch.id_chain]?.applyPatch(patch: patch)
            textElements.objectWillChange.send()
        }

        func handleTextDelete(patch: AnyDeletePatch) {
//            self.requestAnimationFrame { [weak self] in
            self.textElements.remove(id: patch.id_chain)
//            }
        }

        func handleFrameCreate(patch: AnyCreatePatch) {
            frameElements.add(element: FrameElement.makeDefault(id_chain: patch.id_chain))
        }

        func handleFrameUpdate(patch: FrameUpdatePatch) {
            frameElements.elements[patch.id_chain]?.applyPatch(patch: patch)
            frameElements.objectWillChange.send()
        }

        func handleFrameDelete(patch: AnyDeletePatch) {
            frameElements.remove(id: patch.id_chain)
        }

//        let buffer = try! FlexBufferBuilder.encodeMap { builder in
//            builder.add("id_chain", patch.id_chain)
//            builder.addVector("image_data") { imageBuilder in
//                for byte in imageData {
//                    imageBuilder.add(Int(byte))
//                }
//            }
//        }

        func printAllFilesInBundle() {
            let bundleURL = Bundle.main.bundleURL

            do {
                let resourceURLs = try FileManager.default.contentsOfDirectory(at: bundleURL, includingPropertiesForKeys: nil, options: [])
                for url in resourceURLs {
                    print(url.lastPathComponent)
                }
            } catch {
                print("Error: \(error)")
            }
        }


        func handleImageLoad(patch: ImageLoadPatch) {
            Task {
                do {
                    let path = patch.path!
                    let url = URL(fileURLWithPath: path)
                    let fileNameWithExtension = url.lastPathComponent
                    let fileExtension = url.pathExtension
                    let fileName = String(fileNameWithExtension.prefix(fileNameWithExtension.count - fileExtension.count - 1))

                    guard let bundleURL = Bundle.main.url(forResource: fileName, withExtension: fileExtension) else {
                        throw NSError(domain: "", code: 100, userInfo: [NSLocalizedDescriptionKey : "Image file not found in bundle"])
                    }

                    guard let image = NSImage(contentsOf: bundleURL) else {
                        throw NSError(domain: "", code: 101, userInfo: [NSLocalizedDescriptionKey : "Could not create NSImage from data"])
                    }

                    guard let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
                        throw NSError(domain: "", code: 102, userInfo: [NSLocalizedDescriptionKey : "Could not create CGImage from NSImage"])
                    }

                    let width = cgImage.width
                    let height = cgImage.height
                    let bitsPerComponent = cgImage.bitsPerComponent
                    let bytesPerRow = cgImage.bytesPerRow
                    let totalBytes = height * bytesPerRow

                    let colorSpace = CGColorSpaceCreateDeviceRGB()
                    let bitmapInfo = CGImageAlphaInfo.premultipliedLast.rawValue | CGBitmapInfo.byteOrder32Big.rawValue

                    guard let context = CGContext(data: nil, width: width, height: height, bitsPerComponent: bitsPerComponent, bytesPerRow: bytesPerRow, space: colorSpace, bitmapInfo: bitmapInfo) else {
                        throw NSError(domain: "", code: 103, userInfo: [NSLocalizedDescriptionKey : "Could not create CGContext"])
                    }

                    context.draw(cgImage, in: CGRect(x: 0, y: 0, width: width, height: height))

                    guard let data = context.data else {
                        throw NSError(domain: "", code: 104, userInfo: [NSLocalizedDescriptionKey : "Could not retrieve pixel data from context"])
                    }

                    let byteBuffer = data.assumingMemoryBound(to: UInt8.self)

                    let id_chain : FlxbValueVector = FlxbValueVector.init(values: patch.id_chain.map { (number) -> FlxbValue in
                        return number as FlxbValue
                    })
                    let raw_pointer_uint = UInt(bitPattern: byteBuffer)

                    let buffer = try! FlexBufferBuilder.encode(
                        [ "Image": [ "Reference": [
                            "id_chain": id_chain,
                            "image_data": raw_pointer_uint,
                            "image_data_length": totalBytes,
                            "width": width,
                            "height": height,
                        ] as FlxbValueMap] as FlxbValueMap ] as FlxbValueMap)

                        buffer.data.withUnsafeBytes { ptr in
                            var ffi_container = InterruptBuffer(data_ptr: ptr.baseAddress!, length: UInt64(ptr.count))
                            withUnsafePointer(to: &ffi_container) { ffi_container_ptr in
                                pax_interrupt(PaxEngineContainer.paxEngineContainer!, ffi_container_ptr)
                            }
                        }
                } catch {
                    print("Failed to load image data: \(error)")
                }
            }

        }



        func processNativeMessageQueue(queue: NativeMessageQueue) {

            let buffer = UnsafeBufferPointer<UInt8>(start: queue.data_ptr!, count: Int(queue.length))
            let root = FlexBuffer.decode(data: Data.init(buffer: buffer))!

            root["messages"]?.asVector?.makeIterator().forEach( { message in

                let textCreateMessage = message["TextCreate"]
                if textCreateMessage != nil {
                    handleTextCreate(patch: AnyCreatePatch(fb: textCreateMessage!))
                }

                let textUpdateMessage = message["TextUpdate"]
                if textUpdateMessage != nil {
                    handleTextUpdate(patch: TextUpdatePatch(fb: textUpdateMessage!))
                }

                let textDeleteMessage = message["TextDelete"]
                if textDeleteMessage != nil {
                    handleTextDelete(patch: AnyDeletePatch(fb: textDeleteMessage!))
                }

                let frameCreateMessage = message["FrameCreate"]
                if frameCreateMessage != nil {
                    handleFrameCreate(patch: AnyCreatePatch(fb: frameCreateMessage!))
                }

                let frameUpdateMessage = message["FrameUpdate"]
                if frameUpdateMessage != nil {
                    handleFrameUpdate(patch: FrameUpdatePatch(fb: frameUpdateMessage!))
                }

                let frameDeleteMessage = message["FrameDelete"]
                if frameDeleteMessage != nil {
                    handleFrameDelete(patch: AnyDeletePatch(fb: frameDeleteMessage!))
                }

                let imageLoadMessage = message["ImageLoad"]
                if imageLoadMessage != nil {
                    handleImageLoad(patch: ImageLoadPatch(fb: imageLoadMessage!))
                }

                //^ Add new message-receive handlers here ^
            })

        }

        override func scrollWheel(with event: NSEvent){
            let deltaX = event.scrollingDeltaX
            let deltaY = -event.scrollingDeltaY
            let x = event.locationInWindow.x;
            let y = event.locationInWindow.y;
            let json = String(format: "{\"Scroll\": {\"x\": %f, \"y\": %f, \"delta_x\": %f, \"delta_y\": %f} }", x, y, deltaX, deltaY);
            let buffer = try! FlexBufferBuilder.fromJSON(json)

            //Send `Scroll` interrupt
            buffer.data.withUnsafeBytes({ptr in
                var ffi_container = InterruptBuffer( data_ptr: ptr.baseAddress!, length: UInt64(ptr.count) )
                withUnsafePointer(to: &ffi_container) {ffi_container_ptr in
                    pax_interrupt(PaxEngineContainer.paxEngineContainer!, ffi_container_ptr)
                }
            })
        }

    }
}
