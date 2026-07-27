import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/foundation.dart';

import '../model/video_frame.dart';
import 'connection_service.dart';
import 'h264_decoder_service.dart';
import 'quic_transport_service.dart';

enum TransportType { websocket, quic }

class VideoPipelineService extends ChangeNotifier {
  final ConnectionService _wsService;
  final H264DecoderService _h264Decoder;
  final QuicTransportService _quicService;

  TransportType _transport = TransportType.websocket;
  VideoCodec _activeCodec = VideoCodec.jpeg;
  bool _h264Available = false;

  Uint8List? _currentFrame;
  int _frameWidth = 0;
  int _frameHeight = 0;
  int _currentDisplay = 0;
  int _totalDisplays = 1;
  double _fps = 0;

  StreamSubscription<Uint8List>? _quicFrameSub;

  VideoPipelineService(this._wsService, this._h264Decoder, this._quicService) {
    _wsService.addListener(_onWsUpdate);

    _h264Decoder.addListener(() {
      if (_activeCodec == VideoCodec.h264 && _h264Decoder.decodedFrame != null) {
        _currentFrame = _h264Decoder.decodedFrame;
        notifyListeners();
      }
    });
  }

  TransportType get transport => _transport;
  VideoCodec get activeCodec => _activeCodec;
  bool get h264Available => _h264Available;
  Uint8List? get currentFrame => _currentFrame;
  int get frameWidth => _frameWidth;
  int get frameHeight => _frameHeight;
  int get currentDisplay => _currentDisplay;
  int get totalDisplays => _totalDisplays;
  double get fps => _fps;

  Future<void> switchToH264() async {
    if (_h264Available) return;

    final ok = await _h264Decoder.initDecoder(_frameWidth, _frameHeight);
    if (!ok) {
      if (kDebugMode) debugPrint('H.264 decoder unavailable, staying on JPEG');
      return;
    }

    _activeCodec = VideoCodec.h264;
    _h264Available = true;
    notifyListeners();
  }

  Future<void> switchToQuic(String host, int port) async {
    final ok = await _quicService.connect(host, port);
    if (!ok) return;

    await _quicFrameSub?.cancel();
    _quicFrameSub = _quicService.frameStream.listen((data) {
      _handleH264Data(data);
    });

    _transport = TransportType.quic;
    _activeCodec = VideoCodec.h264;
    notifyListeners();
  }

  void _onWsUpdate() {
    if (_transport == TransportType.quic) return;

    final frame = _wsService.currentFrame;
    if (frame != null) {
      final isH264 = frame.length >= 4 && frame[0] == 0x00 && frame[1] == 0x00 &&
          (frame[2] == 0x00 && frame[3] == 0x01 || frame[2] == 0x01);

      if (isH264 && _activeCodec == VideoCodec.h264) {
        _handleH264Data(frame);
      } else {
        _currentFrame = frame;
        _activeCodec = VideoCodec.jpeg;
      }
    }

    _currentDisplay = _wsService.currentDisplay;
    _totalDisplays = _wsService.totalDisplays;
    _fps = _wsService.fps;
    notifyListeners();
  }

  void _handleH264Data(Uint8List data) {
    _h264Decoder.decodeFrame(data);
  }

  @override
  void dispose() {
    _wsService.removeListener(_onWsUpdate);
    _quicFrameSub?.cancel();
    super.dispose();
  }
}
