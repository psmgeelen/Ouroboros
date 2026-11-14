import 'dart:io';
import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ouroboros/backup.dart';
import 'package:ouroboros/export.dart';
import 'package:ouroboros/model.dart';
import 'package:ouroboros/partner/vault.dart';
import 'package:ouroboros/reminders.dart';
import 'package:ouroboros/screens.dart';
import 'package:ouroboros/storage.dart';
import 'package:ouroboros/theme.dart';

void main() {
  test('forest green seed is #228B22', () {
    expect(forestGreen, const Color(0xFF228B22));
  });

  testWidgets('mode choice is shown', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: ouroborosTheme(),
        home: ModeChooser(onTracker: () {}, onPartner: () {}),
      ),
    );
    expect(find.byKey(const Key('choose-tracker')), findsOneWidget);
    expect(find.byKey(const Key('choose-partner')), findsOneWidget);
    expect(find.text('Export'), findsNothing);
  });

  testWidgets('account code is 16 digits and has no share action', (
    tester,
  ) async {
    final code = generateAccountCode(Random(1));
    await tester.pumpWidget(MaterialApp(home: AccountCodeScreen(code: code)));
    expect(RegExp(r'^\d{16}$').hasMatch(code), isTrue);
    expect(find.byKey(const Key('account-code')), findsOneWidget);
    expect(find.text('Share'), findsNothing);
    expect(find.byIcon(Icons.share), findsNothing);
  });

  testWidgets('partner mode has no export button', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(home: PartnerHome(ended: false, phase: 'period')),
    );
    expect(find.byKey(const Key('export-log')), findsNothing);
  });

  testWidgets('empty calendar and a logged day', (tester) async {
    await tester.pumpWidget(const MaterialApp(home: CalendarView(dates: [])));
    expect(find.text('No periods logged'), findsOneWidget);
    await tester.pumpWidget(
      const MaterialApp(home: CalendarView(dates: ['2026-01-01'])),
    );
    expect(find.text('2026-01-01'), findsOneWidget);
  });

  testWidgets('notes round-trip through the form', (tester) async {
    String? saved;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: PeriodForm(onSave: (notes) => saved = notes)),
      ),
    );
    await tester.enterText(find.byKey(const Key('notes-field')), 'cramps');
    await tester.tap(find.byKey(const Key('save-period')));
    expect(saved, 'cramps');
  });

  testWidgets('disclaimer and prompt are visible when due', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: PredictionPanel(low: 26, high: 30, promptDue: true),
      ),
    );
    expect(find.byKey(const Key('disclaimer')), findsOneWidget);
    expect(find.text(wellnessDisclaimer), findsOneWidget);
    expect(find.byKey(const Key('prediction-prompt')), findsOneWidget);
  });

  testWidgets('confirm button is what uploads', (tester) async {
    var uploads = 0;
    await tester.pumpWidget(
      MaterialApp(
        home: PairingScreen(shortCode: '123456', onConfirm: () => uploads += 1),
      ),
    );
    expect(find.byKey(const Key('short-code')), findsOneWidget);
    await tester.tap(find.byKey(const Key('confirm-upload')));
    expect(uploads, 1);
  });

  testWidgets('ended partner screen has no cycle text', (tester) async {
    await tester.pumpWidget(const MaterialApp(home: PartnerHome(ended: true)));
    expect(find.text('Access ended'), findsOneWidget);
    expect(find.text('period'), findsNothing);
  });

  testWidgets('export button is only on the tracker and asks first', (
    tester,
  ) async {
    var confirmed = false;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: TrackerHome(
            dates: const [],
            low: 26,
            high: 30,
            promptDue: false,
            shortCode: '123456',
            onSaveNotes: (_) {},
            onConfirmUpload: () {},
            onExport: () async {
              confirmed = true;
            },
          ),
        ),
      ),
    );
    expect(find.byKey(const Key('export-log')), findsOneWidget);
    await tester.tap(find.byKey(const Key('export-log')));
    await tester.pump();
    expect(confirmed, isTrue);
  });

  test('regular history stays near day 28', () {
    final model = CycleModel();
    for (var i = 0; i < 6; i++) {
      model.observeCycle(28);
    }
    final window = model.nextWindow();
    expect((model.cycleMean - 28).abs() < 1, isTrue);
    expect(window.$2 - window.$1 > 0, isTrue);
  });

  test('not yet does not shrink variance to zero', () {
    final model = CycleModel()..observeCycle(28);
    final before = model.cycleVar;
    model.feedback(FeedbackAnswer.notYet);
    expect(model.cycleVar >= before, isTrue);
    expect(model.cycleVar > 0, isTrue);
  });

  test('single cycle has non-zero width', () {
    final model = CycleModel()..observeCycle(30);
    final window = model.nextWindow();
    expect(window.$2 - window.$1 > 0, isTrue);
  });

  test('encrypted log round-trips and rejects a missing key', () async {
    final dir = await Directory.systemTemp.createTemp('ouroboros-log');
    final file = File('${dir.path}/tracker.bin');
    final db = TrackerDatabase(file: file, key: List<int>.filled(16, 7));
    await db.upsert(
      PeriodRecord(
        start: '2026-01-01',
        end: '2026-01-05',
        flow: 'medium',
        notes: 'cramps',
      ),
    );
    final raw = await file.readAsBytes();
    expect(
      String.fromCharCodes(raw.take(16)).contains('SQLite format 3'),
      isFalse,
    );
    expect(String.fromCharCodes(raw).contains('cramps'), isFalse);
    final opened = TrackerDatabase(file: file, key: List<int>.filled(16, 7));
    await opened.load();
    expect(opened.periods.single.notes, 'cramps');
    await expectLater(
      TrackerDatabase(file: file, key: List<int>.filled(16, 3)).load(),
      throwsA(anything),
    );
  });

  test(
    'export contains the period and account code but not partner ciphertext',
    () {
      const planted = 'partner-ciphertext-blob';
      final encoded = exportTracker(
        directory: '/data/tracker',
        accountCode: '1234567890123456',
        periods: [
          PeriodRecord(
            start: '2026-01-01',
            end: '2026-01-05',
            flow: 'medium',
            notes: 'cramps',
          ),
        ],
        modelSummary: 'cycle mean 28',
      );
      expect(encoded.contains('1234567890123456'), isTrue);
      expect(encoded.contains('2026-01-01'), isTrue);
      expect(encoded.contains(planted), isFalse);
      expect(
        () => exportTracker(
          directory: '/data/partner/cache',
          accountCode: '1234567890123456',
          periods: const [],
          modelSummary: '',
        ),
        throwsStateError,
      );
    },
  );

  test('partner sandbox keeps no epoch key after lock', () async {
    final dir = await Directory.systemTemp.createTemp('ouroboros-partner');
    final vault = PartnerVault(dir);
    vault.show(phase: 'period', trackerFingerprint: 'abcd');
    await vault.persistCiphertext(const [1, 2, 3, 4]);
    vault.lock();
    await vault.wipe();
    await vault.applyFetch(null);
    expect(vault.phaseLabel, isNull);
    expect(File('${dir.path}/cache.bin').existsSync(), isFalse);
    await for (final entity in dir.list(recursive: true)) {
      if (entity is File) {
        final bytes = await entity.readAsBytes();
        expect(String.fromCharCodes(bytes).contains('epoch-key'), isFalse);
        expect(String.fromCharCodes(bytes).contains('period'), isFalse);
      }
    }
  });

  test('sharing off keeps tracker reminders', () {
    final plan = ReminderPlan(
      trackerLocal: ['period window'],
      partnerFacing: ['partner wake'],
    );
    plan.setPartnerSharing(false);
    expect(plan.trackerLocal, ['period window']);
    expect(plan.partnerFacing, isEmpty);
  });

  test('revoked partnership suppresses phase text', () {
    expect(
      partnerPhaseText(sealedKeyPresent: false, phaseLabel: 'period'),
      isNull,
    );
  });

  test('android backup is disabled and ios exclusion lists both sandboxes', () {
    final manifest = File(
      'android/app/src/main/AndroidManifest.xml',
    ).readAsStringSync();
    expect(manifest.contains('android:allowBackup="false"'), isTrue);
    final main = File('lib/main.dart').readAsStringSync();
    expect(main.contains('excludeSandboxesFromBackup'), isTrue);
    final seen = <String>[];
    excludeSandboxesFromBackup(const ['tracker', 'partner'], seen.add);
    expect(seen, ['tracker', 'partner']);
  });
}
