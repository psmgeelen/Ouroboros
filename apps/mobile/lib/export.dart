import 'dart:convert';

import 'storage.dart';

class TrackerExport {
  TrackerExport({
    required this.accountCode,
    required this.periods,
    required this.modelSummary,
  });

  final String accountCode;
  final List<PeriodRecord> periods;
  final String modelSummary;

  String toJson() {
    return jsonEncode({
      'account_code': accountCode,
      'periods': periods.map((period) => period.toJson()).toList(),
      'model_summary': modelSummary,
    });
  }
}

/// Refuse a path inside the partner sandbox.
void exportRefusesPartnerDir(String directory) {
  final parts = directory.split(RegExp(r'[\\/]'));
  if (parts.contains('partner')) {
    throw StateError('partner directory cannot be exported');
  }
}

String exportTracker({
  required String directory,
  required String accountCode,
  required List<PeriodRecord> periods,
  required String modelSummary,
}) {
  exportRefusesPartnerDir(directory);
  return TrackerExport(
    accountCode: accountCode,
    periods: periods,
    modelSummary: modelSummary,
  ).toJson();
}
