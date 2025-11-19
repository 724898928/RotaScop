import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'screens/remote_screen.dart';
import 'services/connection_service.dart';
import 'services/sensor_service.dart';

class MultiScreenRemoteApp extends StatelessWidget {
  const MultiScreenRemoteApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider(create: (_) => ConnectionService()),
        ChangeNotifierProvider(create: (_) => SensorService()),
      ],
      child: MaterialApp(
        title: 'Multi-Screen Remote',
        theme: ThemeData(
          colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
          useMaterial3: true,
        ),
        home: const RemoteScreen(),
      ),
    );
  }
}