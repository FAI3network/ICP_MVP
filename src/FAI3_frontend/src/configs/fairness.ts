const fairnessConfig = {
  SPD: {
    label: "Statistical Parity Difference",
    color: "#2563eb",
    description: "The statistical parity difference measures the difference in the positive outcome rates between the unprivileged group and the privileged group.",
    footer: {
      unfair: "SPD significantly different from 0 (e.g., -0.4 or 0.4)",
      fair: "SPD close to 0 (e.g., -0.1 to 0.1)",
    },
    fairRange: [-0.1, 0.1],
    unfairRange: [-0.4, 0.4],
    key: "average.SPD",
  },
  DI: {
    label: "Disparate Impact",
    color: "#60a5fa",
    description: "Disparate impact compares the ratio of the positive outcome rates between the unprivileged group and the privileged group.",
    footer: {
      unfair: "DI significantly different from 1 (e.g., less than 0.8 or greater than 1.25)",
      fair: "DI close to 1 (e.g., 0.8 to 1.25)",
    },
    fairRange: [0.8, 1.25],
    unfairRange: [0.8, 1.25],
    key: "average.DI",
  },
  AOD: {
    label: "Average Odds Difference",
    color: "#10b981",
    description: "The average odds difference measures the difference in false positive rates and true positive rates between the unprivileged group and the privileged group.",
    footer: {
      fair: "AOD close to 0 (e.g., -0.1 to 0.1)",
      unfair: "AOD significantly different from 0 (e.g., -0.2 or 0.2)",
    },
    fairRange: [-0.1, 0.1],
    unfairRange: [-0.2, 0.2],
    key: "average.AOD",
  },
  EOD: {
    label: "Equal Opportunity Difference",
    color: "#f97316",
    description: "The equal opportunity difference measures the difference in true positive rates between the unprivileged group and the privileged group.",
    footer: {
      fair: "EOD close to 0 (e.g., -0.1 to 0.1)",
      unfair: "EOD significantly different from 0 (e.g., -0.2 or 0.2)",
    },
    unfairRange: [-0.2, 0.2],
    fairRange: [-0.1, 0.1],
    key: "average.EOD",
  },
};

const prefixedFairnessConfig = (prefix: string | undefined) => ({
  SPD: {
    label: "Statistical Parity Difference",
    color: "#2563eb",
    description: "The statistical parity difference measures the difference in the positive outcome rates between the unprivileged group and the privileged group.",
    footer: {
      unfair: "SPD significantly different from 0 (e.g., -0.4 or 0.4)",
      fair: "SPD close to 0 (e.g., -0.1 to 0.1)",
    },
    fairRange: [-0.1, 0.1],
    unfairRange: [-0.4, 0.4],
    key: prefix + "." + "SPD",
  },
  DI: {
    label: "Disparate Impact",
    color: "#60a5fa",
    description: "Disparate impact compares the ratio of the positive outcome rates between the unprivileged group and the privileged group.",
    footer: {
      unfair: "DI significantly different from 1 (e.g., less than 0.8 or greater than 1.25)",
      fair: "DI close to 1 (e.g., 0.8 to 1.25)",
    },
    fairRange: [0.8, 1.25],
    unfairRange: [0.8, 1.25],
    key: prefix + "." + "DI",
  },
  AOD: {
    label: "Average Odds Difference",
    color: "#10b981",
    description: "The average odds difference measures the difference in false positive rates and true positive rates between the unprivileged group and the privileged group.",
    footer: {
      fair: "AOD close to 0 (e.g., -0.1 to 0.1)",
      unfair: "AOD significantly different from 0 (e.g., -0.2 or 0.2)",
    },
    fairRange: [-0.1, 0.1],
    unfairRange: [-0.2, 0.2],
    key: prefix + "." + "AOD",
  },
  EOD: {
    label: "Equal Opportunity Difference",
    color: "#f97316",
    description: "The equal opportunity difference measures the difference in true positive rates between the unprivileged group and the privileged group.",
    footer: {
      fair: "EOD close to 0 (e.g., -0.1 to 0.1)",
      unfair: "EOD significantly different from 0 (e.g., -0.2 or 0.2)",
    },
    unfairRange: [-0.2, 0.2],
    fairRange: [-0.1, 0.1],
    key: prefix + "." + "EOD",
  },
});

export { fairnessConfig, prefixedFairnessConfig };
