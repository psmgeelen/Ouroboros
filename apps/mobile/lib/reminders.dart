class ReminderPlan {
  ReminderPlan({List<String>? trackerLocal, List<String>? partnerFacing})
    : trackerLocal = List<String>.of(trackerLocal ?? const []),
      partnerFacing = List<String>.of(partnerFacing ?? const []);

  final List<String> trackerLocal;
  final List<String> partnerFacing;

  void setPartnerSharing(bool enabled) {
    if (!enabled) {
      partnerFacing.clear();
    }
  }
}

String? partnerPhaseText({
  required bool sealedKeyPresent,
  required String phaseLabel,
}) {
  if (!sealedKeyPresent) {
    return null;
  }
  return phaseLabel;
}
