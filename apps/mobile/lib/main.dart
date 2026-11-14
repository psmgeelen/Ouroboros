import 'dart:math';

import 'package:flutter/material.dart';

import 'backup.dart';
import 'model.dart';
import 'screens.dart';
import 'theme.dart';

void main() {
  excludeSandboxesFromBackup(const ['tracker', 'partner'], (_) {});
  runApp(const OuroborosApp());
}

class OuroborosApp extends StatefulWidget {
  const OuroborosApp({super.key});

  @override
  State<OuroborosApp> createState() => _OuroborosAppState();
}

class _OuroborosAppState extends State<OuroborosApp> {
  String? _mode;
  late final String _code = generateAccountCode(Random.secure());
  bool _codeSeen = false;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Ouroboros',
      theme: ouroborosTheme(),
      darkTheme: ouroborosTheme(brightness: Brightness.dark),
      home: Scaffold(
        appBar: AppBar(title: const Text('Ouroboros')),
        body: Padding(padding: const EdgeInsets.all(16), child: _body()),
      ),
    );
  }

  Widget _body() {
    if (_mode == null) {
      return ModeChooser(
        onTracker: () => setState(() => _mode = 'tracker'),
        onPartner: () => setState(() => _mode = 'partner'),
      );
    }
    if (_mode == 'tracker' && !_codeSeen) {
      return Column(
        children: [
          AccountCodeScreen(code: _code),
          TextButton(
            onPressed: () => setState(() => _codeSeen = true),
            child: const Text('I wrote it down'),
          ),
        ],
      );
    }
    if (_mode == 'partner') {
      return const PartnerHome(ended: false, phase: '', fingerprint: '');
    }
    final model = CycleModel();
    final window = model.nextWindow();
    return TrackerHome(
      dates: const [],
      low: window.$1,
      high: window.$2,
      promptDue: model.promptDue(model.cycleMean),
      shortCode: '000000',
      onSaveNotes: (_) {},
      onConfirmUpload: () {},
      onExport: () async {},
    );
  }
}
