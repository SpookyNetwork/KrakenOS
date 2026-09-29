use crate::ast::policy::DoctrinePolicy;
use crate::ir::graph::PolicyGraph;
use petgraph::graph::DiGraph;

pub fn lower_policy(policy: DoctrinePolicy) -> PolicyGraph {
    let mut graph = DiGraph::new();

    // Map AST to policy nodes
    let system_node = graph.add_node(format!("system:{}", policy.system));

    if let Some(mem) = policy.policy.memory {
        let read_node = graph.add_node(format!("memory:read:{}", mem.read));
        let write_node = graph.add_node(format!("memory:write:{}", mem.write));
        graph.add_edge(system_node, read_node, 1.0);
        graph.add_edge(system_node, write_node, 1.0);
    }

    if let Some(agents) = policy.policy.agents {
        for (agent_name, agent_pol) in agents {
            let agent_node = graph.add_node(format!("agent:{}", agent_name));
            let budget_node = graph.add_node(format!("agent:{}:budget:{}", agent_name, agent_pol.budget));
            let net_node = graph.add_node(format!("agent:{}:network:{}", agent_name, agent_pol.network));
            
            graph.add_edge(system_node, agent_node, 1.0);
            graph.add_edge(agent_node, budget_node, 1.0);
            graph.add_edge(agent_node, net_node, 1.0);
        }
    }

    if let Some(exec) = policy.policy.execution {
        let exec_node = graph.add_node(format!("execution:external:{}", exec.external_calls));
        graph.add_edge(system_node, exec_node, 1.0);
    }

    PolicyGraph { graph }
}
