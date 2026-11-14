import 'package:material_ui/material_ui.dart';

import 'model.dart';

class ModeChooser extends StatelessWidget {
  const ModeChooser({
    super.key,
    required this.onTracker,
    required this.onPartner,
  });

  final VoidCallback onTracker;
  final VoidCallback onPartner;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text('Ouroboros', style: Theme.of(context).textTheme.headlineMedium),
        const SizedBox(height: 16),
        FilledButton(
          key: const Key('choose-tracker'),
          onPressed: onTracker,
          child: const Text('Tracker'),
        ),
        const SizedBox(height: 8),
        TextButton(
          key: const Key('choose-partner'),
          onPressed: onPartner,
          child: const Text('Partner'),
        ),
      ],
    );
  }
}

class AccountCodeScreen extends StatelessWidget {
  const AccountCodeScreen({super.key, required this.code});

  final String code;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(code, key: const Key('account-code')),
        const Text('Write this code down. It is shown once.'),
      ],
    );
  }
}

class CalendarView extends StatelessWidget {
  const CalendarView({super.key, required this.dates});

  final List<String> dates;

  @override
  Widget build(BuildContext context) {
    if (dates.isEmpty) {
      return const Text('No periods logged');
    }
    return Column(children: [for (final date in dates) Text(date)]);
  }
}

class PeriodForm extends StatefulWidget {
  const PeriodForm({super.key, required this.onSave});

  final ValueChanged<String> onSave;

  @override
  State<PeriodForm> createState() => _PeriodFormState();
}

class _PeriodFormState extends State<PeriodForm> {
  final _notes = TextEditingController();

  @override
  void dispose() {
    _notes.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        TextField(
          key: const Key('notes-field'),
          controller: _notes,
          decoration: const InputDecoration(labelText: 'Notes'),
        ),
        FilledButton(
          key: const Key('save-period'),
          onPressed: () => widget.onSave(_notes.text),
          child: const Text('Save period'),
        ),
      ],
    );
  }
}

class PredictionPanel extends StatelessWidget {
  const PredictionPanel({
    super.key,
    required this.low,
    required this.high,
    required this.promptDue,
  });

  final double low;
  final double high;
  final bool promptDue;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text('${low.round()}–${high.round()} days'),
        Text(wellnessDisclaimer, key: const Key('disclaimer')),
        if (promptDue)
          const Text('Did your period start?', key: Key('prediction-prompt')),
      ],
    );
  }
}

class PairingScreen extends StatelessWidget {
  const PairingScreen({
    super.key,
    required this.shortCode,
    required this.onConfirm,
  });

  final String shortCode;
  final VoidCallback onConfirm;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        const Text('Compare this code with your partner'),
        Text(shortCode, key: const Key('short-code')),
        FilledButton(
          key: const Key('confirm-upload'),
          onPressed: onConfirm,
          child: const Text('Confirm and upload'),
        ),
      ],
    );
  }
}

class TrackerHome extends StatelessWidget {
  const TrackerHome({
    super.key,
    required this.dates,
    required this.low,
    required this.high,
    required this.promptDue,
    required this.shortCode,
    required this.onSaveNotes,
    required this.onConfirmUpload,
    required this.onExport,
  });

  final List<String> dates;
  final double low;
  final double high;
  final bool promptDue;
  final String shortCode;
  final ValueChanged<String> onSaveNotes;
  final VoidCallback onConfirmUpload;
  final Future<void> Function() onExport;

  @override
  Widget build(BuildContext context) {
    return SecureScreen(
      child: ListView(
        children: [
          CalendarView(dates: dates),
          PeriodForm(onSave: onSaveNotes),
          PredictionPanel(low: low, high: high, promptDue: promptDue),
          PairingScreen(shortCode: shortCode, onConfirm: onConfirmUpload),
          FilledButton(
            key: const Key('export-log'),
            onPressed: () => onExport(),
            child: const Text('Export'),
          ),
        ],
      ),
    );
  }
}

class PartnerHome extends StatelessWidget {
  const PartnerHome({
    super.key,
    required this.ended,
    this.phase,
    this.fingerprint,
  });

  final bool ended;
  final String? phase;
  final String? fingerprint;

  @override
  Widget build(BuildContext context) {
    return SecureScreen(
      child: ended
          ? const Text('Access ended')
          : Column(children: [Text(phase ?? ''), Text(fingerprint ?? '')]),
    );
  }
}

class SecureScreen extends StatelessWidget {
  const SecureScreen({super.key, required this.child});

  final Widget child;
  static bool active = false;

  @override
  Widget build(BuildContext context) {
    active = true;
    return child;
  }
}
