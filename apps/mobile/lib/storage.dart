import 'dart:convert';
import 'dart:io';

class PeriodRecord {
  PeriodRecord({
    required this.start,
    required this.end,
    required this.flow,
    required this.notes,
  });

  final String start;
  final String end;
  final String flow;
  final String notes;

  Map<String, String> toJson() => {
    'start': start,
    'end': end,
    'flow': flow,
    'notes': notes,
  };

  static PeriodRecord fromJson(Map<String, dynamic> json) {
    return PeriodRecord(
      start: json['start'] as String,
      end: json['end'] as String,
      flow: json['flow'] as String,
      notes: json['notes'] as String,
    );
  }
}

/// Encrypted tracker log. The key is not written into the file.
class TrackerDatabase {
  TrackerDatabase({required this.file, required this.key});

  final File file;
  final List<int> key;
  final List<PeriodRecord> periods = [];

  Future<void> load() async {
    if (!await file.exists()) {
      return;
    }
    final sealed = await file.readAsBytes();
    final plain = _open(sealed, key);
    final decoded = jsonDecode(plain) as List<dynamic>;
    periods
      ..clear()
      ..addAll(
        decoded.map(
          (item) => PeriodRecord.fromJson(item as Map<String, dynamic>),
        ),
      );
  }

  Future<void> upsert(PeriodRecord record) async {
    final index = periods.indexWhere((item) => item.start == record.start);
    if (index >= 0) {
      periods[index] = record;
    } else {
      periods.add(record);
    }
    await _write();
  }

  Future<void> _write() async {
    final plain = jsonEncode(periods.map((item) => item.toJson()).toList());
    await file.writeAsBytes(_seal(utf8.encode(plain), key));
  }

  static List<int> _seal(List<int> plain, List<int> key) {
    final out = List<int>.filled(plain.length + 4, 0);
    out[0] = 0x4F;
    out[1] = 0x55;
    out[2] = 0x42;
    out[3] = 0x31;
    for (var i = 0; i < plain.length; i++) {
      out[i + 4] = plain[i] ^ key[i % key.length];
    }
    return out;
  }

  static String _open(List<int> sealed, List<int> key) {
    if (sealed.length < 4 ||
        sealed[0] != 0x4F ||
        sealed[1] != 0x55 ||
        sealed[2] != 0x42) {
      throw StateError('database key rejected');
    }
    final plain = List<int>.generate(
      sealed.length - 4,
      (i) => sealed[i + 4] ^ key[i % key.length],
    );
    return utf8.decode(plain);
  }
}
