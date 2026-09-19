import 'package:flutter/material.dart';

void main() {
  runApp(const DevFlowFlutterApp());
}

class DevFlowFlutterApp extends StatelessWidget {
  const DevFlowFlutterApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'DevFlow Flutter Demo',
      theme: ThemeData.dark(),
      home: const Scaffold(
        body: Center(
          child: Text(
            'DevFlow Flutter Hot Reload Ready',
            style: TextStyle(fontSize: 20, color: Colors.tealAccent),
          ),
        ),
      ),
    );
  }
}
