import { EcmaNodeData, GraphEdgeData } from '../types/engine';

export function generateInitialGraph(nodeCount: number = 28): { nodes: EcmaNodeData[]; edges: GraphEdgeData[] } {
  const nodes: EcmaNodeData[] = [];
  const edges: GraphEdgeData[] = [];

  const labels = [
    "UserIntent_LoanEval", "FinancialRiskMatrix", "CreditScore_Aggregator",
    "CustomerProfile_KYC", "TaxReturnParser", "BankruptcyDetector",
    "IncomeVerification", "CollateralValuation", "DebtToIncomeRatio",
    "EmploymentStability", "AIS_DefenseSentinel", "PromptShield_Vector",
    "ECMA_StemSeed_Alpha", "ECMA_Mature_Core", "ACO_Highway_Hub_01",
    "ACO_Highway_Hub_02", "mRNA_CodonSynthesizer", "ActionExecutor_RiskApprove",
    "ActionExecutor_ManualReview", "ActionExecutor_Reject", "Sinkhorn_MetricNode",
    "Wasserstein_Transport", "Cosine_SIMD_Cluster", "HNSW_NavigableLevel0",
    "HNSW_NavigableLevel1", "Stem_Emergent_Context", "Apoptosis_Candidate_99",
    "Apoptosis_Candidate_42"
  ];

  for (let i = 0; i < nodeCount; i++) {
    let state: EcmaNodeData['state'] = 'Specialized';
    let maturity = 0.85;
    let entropy = 0.05 + Math.random() * 0.15;
    let usage = 200 + Math.floor(Math.random() * 800);

    if (i % 7 === 0) {
      state = 'Stem';
      maturity = 0.15 + Math.random() * 0.2;
      entropy = 0.6 + Math.random() * 0.3;
      usage = Math.floor(Math.random() * 40);
    } else if (i % 5 === 0) {
      state = 'Progenitor';
      maturity = 0.45 + Math.random() * 0.25;
      entropy = 0.25 + Math.random() * 0.25;
      usage = 60 + Math.floor(Math.random() * 120);
    } else if (i >= nodeCount - 2) {
      state = 'Apoptosis';
      maturity = 0.95;
      entropy = 0.85 + Math.random() * 0.1;
      usage = 3;
    }

    nodes.push({
      id: 1000 + i,
      label: labels[i % labels.length] || `ContextNode_${1000 + i}`,
      state,
      usage_count: usage,
      entropy: parseFloat(entropy.toFixed(3)),
      connections: 0,
      maturity: parseFloat(maturity.toFixed(2)),
      x: (Math.random() - 0.5) * 800,
      y: (Math.random() - 0.5) * 600,
      vx: 0,
      vy: 0,
      radius: state === 'Specialized' ? 14 : state === 'Progenitor' ? 11 : state === 'Stem' ? 9 : 8,
      isPathogen: false,
      highlighted: false,
    });
  }

  // Create biological edges with ACO pheromones
  for (let i = 0; i < nodes.length; i++) {
    // Connect to 2-4 nearest or strategic neighbors
    const targetCount = 2 + Math.floor(Math.random() * 3);
    for (let k = 0; k < targetCount; k++) {
      const targetIdx = Math.floor(Math.random() * nodes.length);
      if (targetIdx !== i) {
        const fromId = nodes[i].id;
        const toId = nodes[targetIdx].id;

        // Check if edge already exists
        const exists = edges.some(
          e => (e.from === fromId && e.to === toId) || (e.from === toId && e.to === fromId)
        );

        if (!exists) {
          // Highway edges (high pheromone) between mature/specialized nodes
          const isHighway = nodes[i].state === 'Specialized' && nodes[targetIdx].state === 'Specialized' && Math.random() > 0.4;
          const pheromone = isHighway ? 4.5 + Math.random() * 5.0 : 0.2 + Math.random() * 1.5;

          edges.push({
            from: fromId,
            to: toId,
            weight: parseFloat((0.4 + Math.random() * 0.6).toFixed(2)),
            pheromone: parseFloat(pheromone.toFixed(2)),
            particles: Array.from({ length: isHighway ? 3 : 1 }, () => ({
              progress: Math.random(),
              speed: 0.004 + Math.random() * 0.008,
            })),
          });

          nodes[i].connections++;
          nodes[targetIdx].connections++;
        }
      }
    }
  }

  return { nodes, edges };
}
