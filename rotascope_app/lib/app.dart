import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'screens/remote_screen.dart';
import 'services/connection_service.dart';
import 'services/sensor_service.dart';

class RotascopeApp extends StatelessWidget {
  const RotascopeApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider(create: (_) => ConnectionService()),
        ChangeNotifierProvider(create: (_) => SensorService()),
      ],
      child: MaterialApp(
        title: 'Rotascope',
        theme: ThemeData(
          colorScheme: ColorScheme.fromSeed(
            seedColor: Colors.deepPurple,
            brightness: Brightness.dark,
          ),
          useMaterial3: true,
          scaffoldBackgroundColor: Colors.black,
        ),
        darkTheme: ThemeData(
          colorScheme: ColorScheme.fromSeed(
            seedColor: Colors.deepPurple,
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