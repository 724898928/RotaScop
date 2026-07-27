import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'screens/remote_screen.dart';
import 'services/connection_service.dart';
import 'services/h264_decoder_service.dart';
import 'services/quic_transport_service.dart';
import 'services/sensor_service.dart';
import 'services/video_pipeline_service.dart';

class RotascopeApp extends StatelessWidget {
  const RotascopeApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider(create: (_) => ConnectionService()),
        ChangeNotifierProvider(create: (_) => SensorService()),
        ChangeNotifierProvider(create: (_) => H264DecoderService()),
        ChangeNotifierProvider(create: (_) => QuicTransportService()),
        ChangeNotifierProvider(
          create: (ctx) => VideoPipelineService(
            ctx.read<ConnectionService>(),
            ctx.read<H264DecoderService>(),
            ctx.read<QuicTransportService>(),
          ),
        ),
      ],
      child: MaterialApp(
        title: 'Rotascope',
        theme: ThemeData(
          colorScheme: ColorScheme.fromSeed(
            seedColor: const Color(0xFF32D2C6),
            brightness: Brightness.dark,
          ),
          useMaterial3: true,
          scaffoldBackgroundColor: Colors.black,
        ),
        darkTheme: ThemeData(
          colorScheme: ColorScheme.fromSeed(
            seedColor: const Color(0xFF32D2C6),
            brightness: Brightness.dark,
          ),
          useMaterial3: true,
          scaffoldBackgroundColor: Colors.black,
        ),
        themeMode: ThemeMode.dark,
        home: const RemoteScreen(),
        debugShowCheckedModeBanner: false,
      ),
    );
  }
}
