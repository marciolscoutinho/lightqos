# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# quantum_network.py — Quantum Network — entanglement-based network simulation
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 23-11-2023
# All rights reserved.
# -----------------------------------------------------------------------------

"""
Quantum Network - Distributed Quantum Network

Implements a quantum network with:
- Quantum nodes (QuantumNode)
- Quantum links (QuantumLink)
- Entanglement routing
- Bell-pair purification
- Entanglement swapping
"""

from typing import List, Dict, Optional, Tuple
from dataclasses import dataclass
from enum import Enum
import numpy as np
from collections import defaultdict


class NodeType(Enum):
    """Node type"""
    END_NODE = "end"        # Terminal node (Alice, Bob)
    REPEATER = "repeater"   # Quantum repeater
    ROUTER = "router"       # Quantum router


@dataclass
class EntangledPair:
    """Entangled pair"""
    id: str
    node_a: str
    node_b: str
    fidelity: float
    age: int = 0  # Number of timesteps


@dataclass
class QuantumLink:
    """Quantum link between two nodes"""
    node_a: str
    node_b: str
    distance_km: float
    fidelity: float
    generation_rate: float  # pairs/second
    available: bool = True


class QuantumNode:
    """
    Quantum Node
    
    Can be:
    - Terminal node (Alice, Bob)
    - Quantum repeater
    - Quantum router
    """
    
    def __init__(
        self,
        name: str,
        node_type: NodeType,
        num_qubits: int = 10
    ):
        self.name = name
        self.node_type = node_type
        self.num_qubits = num_qubits
        
        # Stored entangled pairs
        self.entangled_pairs: List[EntangledPair] = []
        
        # Neighbors (connected nodes)
        self.neighbors: List[str] = []
        
        # Statistics
        self.pairs_generated = 0
        self.pairs_consumed = 0
        self.swaps_performed = 0
    
    def add_neighbor(self, neighbor: str):
        """Adds a neighbor"""
        if neighbor not in self.neighbors:
            self.neighbors.append(neighbor)
    
    def store_pair(self, pair: EntangledPair):
        """Stores an entangled pair"""
        if len(self.entangled_pairs) < self.num_qubits:
            self.entangled_pairs.append(pair)
            return True
        return False
    
    def consume_pair(self, pair_id: str) -> Optional[EntangledPair]:
        """Consumes an entangled pair"""
        for i, pair in enumerate(self.entangled_pairs):
            if pair.id == pair_id:
                consumed = self.entangled_pairs.pop(i)
                self.pairs_consumed += 1
                return consumed
        return None
    
    def get_pair_with(self, other_node: str) -> Optional[EntangledPair]:
        """Gets a pair with a specific node"""
        for pair in self.entangled_pairs:
            if pair.node_a == other_node or pair.node_b == other_node:
                return pair
        return None
    
    def age_pairs(self):
        """Ages pairs (fidelity degradation)"""
        for pair in self.entangled_pairs:
            pair.age += 1
            # Exponential degradation
            pair.fidelity *= 0.99  # 1% degradation per timestep


class QuantumNetwork:
    """
    Complete Quantum Network
    
    Manages:
    - Nodes quantums
    - Links quantums
    - Entanglement generation
    - Routing
    - Purification
    - Swapping
    """
    
    def __init__(self):
        self.nodes: Dict[str, QuantumNode] = {}
        self.links: List[QuantumLink] = []
        self.timestep = 0
        
        # Statistics globais
        self.total_pairs_generated = 0
        self.total_swaps = 0
        self.total_purifications = 0
    
    # ========================================================================
    # NETWORK CONSTRUCTION
    # ========================================================================
    
    def add_node(self, name: str, node_type: NodeType, num_qubits: int = 10) -> QuantumNode:
        """Adds a node to the network"""
        node = QuantumNode(name, node_type, num_qubits)
        self.nodes[name] = node
        return node
    
    def add_link(
        self,
        node_a: str,
        node_b: str,
        distance_km: float,
        fidelity: float = 0.95,
        generation_rate: float = 1000.0
    ) -> QuantumLink:
        """
        Adds a quantum link between two nodes
        
        Args:
            node_a, node_b: Node names
            distance_km: Distance in km
            fidelity: Initial pair fidelity
            generation_rate: Taxa de geração (pairs/second)
        """
        # Add neighbors
        self.nodes[node_a].add_neighbor(node_b)
        self.nodes[node_b].add_neighbor(node_a)
        
        link = QuantumLink(
            node_a=node_a,
            node_b=node_b,
            distance_km=distance_km,
            fidelity=fidelity,
            generation_rate=generation_rate
        )
        
        self.links.append(link)
        return link
    
    # ========================================================================
    # ENTANGLEMENT GENERATION
    # ========================================================================
    
    def generate_entanglement(self, node_a: str, node_b: str) -> Optional[EntangledPair]:
        """
        Generates an entangled pair between two adjacent nodes
        
        Returns:
            EntangledPair on success, None on failure
        """
        # Check whether the link exists
        link = self._get_link(node_a, node_b)
        if not link or not link.available:
            return None
        
        # Generate pair
        pair = EntangledPair(
            id=f"pair_{self.total_pairs_generated}",
            node_a=node_a,
            node_b=node_b,
            fidelity=link.fidelity
        )
        
        # Store in both nodes
        if self.nodes[node_a].store_pair(pair) and self.nodes[node_b].store_pair(pair):
            self.total_pairs_generated += 1
            self.nodes[node_a].pairs_generated += 1
            self.nodes[node_b].pairs_generated += 1
            return pair
        
        return None
    
    def _get_link(self, node_a: str, node_b: str) -> Optional[QuantumLink]:
        """Gets the link between two nodes"""
        for link in self.links:
            if (link.node_a == node_a and link.node_b == node_b) or \
               (link.node_a == node_b and link.node_b == node_a):
                return link
        return None
    
    # ========================================================================
    # PURIFICATION
    # ========================================================================
    
    def purify_pairs(
        self,
        pair1: EntangledPair,
        pair2: EntangledPair
    ) -> Optional[EntangledPair]:
        """
        Purifies two low-fidelity pairs into one high-fidelity pair
        
        Protocolo: F_new = F1² + F2² - 2F1²F2²
        
        Args:
            pair1, pair2: Pairs to purify (same nodes)
            
        Returns:
            Purified pair or None on failure
        """
        # Check whether pairs have the same nodes
        if not (pair1.node_a == pair2.node_a and pair1.node_b == pair2.node_b):
            return None
        
        # Purification protocol
        f1 = pair1.fidelity
        f2 = pair2.fidelity
        
        # Purification formula
        f_new = f1**2 + f2**2 - 2 * f1**2 * f2**2
        
        # Success probability
        p_success = f1 * f2 + (1 - f1) * (1 - f2)
        
        if np.random.rand() < p_success:
            # Success: create purified pair
            purified = EntangledPair(
                id=f"purified_{self.total_purifications}",
                node_a=pair1.node_a,
                node_b=pair1.node_b,
                fidelity=min(f_new, 1.0)
            )
            
            # Consume original pairs
            self.nodes[pair1.node_a].consume_pair(pair1.id)
            self.nodes[pair1.node_b].consume_pair(pair1.id)
            self.nodes[pair2.node_a].consume_pair(pair2.id)
            self.nodes[pair2.node_b].consume_pair(pair2.id)
            
            # Store purified pair
            self.nodes[purified.node_a].store_pair(purified)
            self.nodes[purified.node_b].store_pair(purified)
            
            self.total_purifications += 1
            return purified
        
        return None
    
    # ========================================================================
    # ENTANGLEMENT SWAPPING
    # ========================================================================
    
    def entanglement_swapping(
        self,
        pair_ab: EntangledPair,
        pair_bc: EntangledPair,
        middle_node: str
    ) -> Optional[EntangledPair]:
        """
        Performs entanglement swapping
        
        If Alice-Bob are entangled and Bob-Charlie are entangled,
        after swapping: Alice-Charlie become entangled
        
        Args:
            pair_ab: Alice-Bob pair
            pair_bc: Bob-Charlie pair
            middle_node: Intermediate node (Bob)
            
        Returns:
            New Alice-Charlie pair
        """
        # Determine external nodes
        nodes_ab = {pair_ab.node_a, pair_ab.node_b}
        nodes_bc = {pair_bc.node_a, pair_bc.node_b}
        
        if middle_node not in nodes_ab or middle_node not in nodes_bc:
            return None
        
        # External nodes
        external_nodes = (nodes_ab | nodes_bc) - {middle_node}
        if len(external_nodes) != 2:
            return None
        
        node_a, node_c = list(external_nodes)
        
        # Fidelity of the new pair
        f_ab = pair_ab.fidelity
        f_bc = pair_bc.fidelity
        f_ac = f_ab * f_bc  # Simplification
        
        # Create new pair
        swapped_pair = EntangledPair(
            id=f"swapped_{self.total_swaps}",
            node_a=node_a,
            node_b=node_c,
            fidelity=f_ac
        )
        
        # Consume original pairs
        self.nodes[pair_ab.node_a].consume_pair(pair_ab.id)
        self.nodes[pair_ab.node_b].consume_pair(pair_ab.id)
        self.nodes[pair_bc.node_a].consume_pair(pair_bc.id)
        self.nodes[pair_bc.node_b].consume_pair(pair_bc.id)
        
        # Store new pair
        self.nodes[node_a].store_pair(swapped_pair)
        self.nodes[node_c].store_pair(swapped_pair)
        
        self.total_swaps += 1
        self.nodes[middle_node].swaps_performed += 1
        
        return swapped_pair
    
    # ========================================================================
    # ROUTING
    # ========================================================================
    
    def find_path(self, source: str, destination: str) -> Optional[List[str]]:
        """
        Finds a path between two nodes (BFS)
        
        Returns:
            List of nodes in the path or None
        """
        from collections import deque
        
        if source == destination:
            return [source]
        
        visited = {source}
        queue = deque([(source, [source])])
        
        while queue:
            node, path = queue.popleft()
            
            for neighbor in self.nodes[node].neighbors:
                if neighbor == destination:
                    return path + [neighbor]
                
                if neighbor not in visited:
                    visited.add(neighbor)
                    queue.append((neighbor, path + [neighbor]))
        
        return None
    
    def establish_e2e_entanglement(
        self,
        source: str,
        destination: str,
        min_fidelity: float = 0.8
    ) -> Optional[EntangledPair]:
        """
        Establishes end-to-end entanglement
        
        Steps:
        1. Find route
        2. Generate pairs on each link
        3. Swapping at intermediate nodes
        4. Purify if necessary
        
        Returns:
            Final source-destination pair or None
        """
        # 1. Find route
        path = self.find_path(source, destination)
        if not path or len(path) < 2:
            return None
        
        # 2. Generate pairs em cada link
        pairs = []
        for i in range(len(path) - 1):
            pair = self.generate_entanglement(path[i], path[i + 1])
            if not pair:
                return None  # Generation failure
            pairs.append(pair)
        
        # 3. Swapping at intermediate nodes
        current_pair = pairs[0]
        
        for i in range(1, len(pairs)):
            middle_node = path[i]
            next_pair = pairs[i]
            
            current_pair = self.entanglement_swapping(
                current_pair,
                next_pair,
                middle_node
            )
            
            if not current_pair:
                return None  # Swapping failure
        
        # 4. Purify if fidelity is low
        if current_pair.fidelity < min_fidelity:
            # Try to generate another pair and purify
            # (simplification: only return if it was not reached)
            pass
        
        return current_pair
    
    # ========================================================================
    # SIMULATION
    # ========================================================================
    
    def step(self):
        """Advances the simulation by one timestep"""
        self.timestep += 1
        
        # Age pairs
        for node in self.nodes.values():
            node.age_pairs()
    
    def get_statistics(self) -> dict:
        """Returns network statistics"""
        total_pairs = sum(len(node.entangled_pairs) for node in self.nodes.values())
        avg_fidelity = np.mean([
            pair.fidelity
            for node in self.nodes.values()
            for pair in node.entangled_pairs
        ]) if total_pairs > 0 else 0.0
        
        return {
            "timestep": self.timestep,
            "nodes": len(self.nodes),
            "links": len(self.links),
            "stored_pairs": total_pairs,
            "avg_fidelity": avg_fidelity,
            "total_generated": self.total_pairs_generated,
            "total_swaps": self.total_swaps,
            "total_purifications": self.total_purifications,
        }


# ============================================================================
# USAGE EXAMPLE
# ============================================================================

if __name__ == "__main__":
    print("=== Quantum Network Simulation ===\n")
    
    # Create network
    network = QuantumNetwork()
    
    # Add nodes
    alice = network.add_node("Alice", NodeType.END_NODE)
    bob = network.add_node("Bob", NodeType.REPEATER)
    charlie = network.add_node("Charlie", NodeType.END_NODE)
    
    # Add links
    network.add_link("Alice", "Bob", distance_km=50, fidelity=0.9)
    network.add_link("Bob", "Charlie", distance_km=50, fidelity=0.9)
    
    print("Network created:")
    print(f"  Nodes: {list(network.nodes.keys())}")
    print(f"  Links: Alice-Bob, Bob-Charlie\n")
    
    # Estabelecer entanglement end-to-end
    print("Establishing Alice-Charlie entanglement...")
    pair = network.establish_e2e_entanglement("Alice", "Charlie")
    
    if pair:
        print(f"✓ Success!")
        print(f"  ID: {pair.id}")
        print(f"  Fidelity: {pair.fidelity:.3f}\n")
    else:
        print("✗ Failure\n")
    
    # Statistics
    stats = network.get_statistics()
    print("=== Statistics ===")
    print(f"Generated pairs: {stats['total_generated']}")
    print(f"Swaps performed: {stats['total_swaps']}")
    print(f"Stored pairs: {stats['stored_pairs']}")
    print(f"Average fidelity: {stats['avg_fidelity']:.3f}")
