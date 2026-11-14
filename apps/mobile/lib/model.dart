import 'dart:math';

/// Same sentence as `WELLNESS_DISCLAIMER` in the Rust core.
const String wellnessDisclaimer =
    'This is an estimate from your own logs. Ouroboros is not a medical device.';

enum FeedbackAnswer { started, notYet, startedEarlier }

class CycleModel {
  CycleModel({this.cycleMean = 28, this.cycleVar = 4, this.samples = 0});

  double cycleMean;
  double cycleVar;
  int samples;

  void observeCycle(double lengthDays) {
    samples += 1;
    final n = samples.toDouble();
    final delta = lengthDays - cycleMean;
    cycleMean += delta / n;
    final delta2 = lengthDays - cycleMean;
    cycleVar = ((cycleVar * (n - 1)) + delta * delta2) / n;
    if (cycleVar < 1) {
      cycleVar = 1;
    }
  }

  void feedback(FeedbackAnswer answer) {
    switch (answer) {
      case FeedbackAnswer.started:
        observeCycle(cycleMean);
      case FeedbackAnswer.notYet:
        cycleMean += 1;
        cycleVar = max(cycleVar + 1, 1);
      case FeedbackAnswer.startedEarlier:
        cycleMean = max(cycleMean - 1, 1);
        cycleVar = max(cycleVar + 0.5, 1);
    }
  }

  (double, double) nextWindow() {
    final width = max(sqrt(cycleVar), 1);
    return (cycleMean - width, cycleMean + width);
  }

  bool promptDue(double day) {
    final window = nextWindow();
    return day >= window.$1 && day <= window.$2;
  }
}

String generateAccountCode(Random random) {
  return List.generate(16, (_) => random.nextInt(10).toString()).join();
}
