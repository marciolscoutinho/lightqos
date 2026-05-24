# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# quantum_network.py — Quantum Network — multi-node entanglement distribution
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 26-05-2023
# All rights reserved.
# -----------------------------------------------------------------------------

"""
Example: Distributed Quantum Network with LightQOS
Demonstrates quantum communication between nodes using EFAL + EMF
"""

try :
    from lightqos import QuantumNetwork ,QuantumNode ,QuantumCircuit 
    from lightqos import TemporalContract ,FidelityContract 
    from lightqos .protocols import BellPairDistribution ,Teleportation 
    LIGHTQOS_AVAILABLE =True 
except ImportError :
    print ("⚠️  LightQOS not installed. Demo mode.")
    LIGHTQOS_AVAILABLE =False 

import numpy as np 


# ============================================================================
# EXEMPLO 1: Network de 3 Nodes com Distribution de Entanglement
# ============================================================================

print ("="*80 )
print ("EXAMPLE 1: 3-Node Quantum Network - EPR Distribution")
print ("="*80 )

if LIGHTQOS_AVAILABLE :
# Create network
    network =QuantumNetwork (name ="qnet_demo")

    # Add nodes
    alice =QuantumNode (name ="Alice",num_qubits =3 ,position =(0 ,0 ))
    bob =QuantumNode (name ="Bob",num_qubits =3 ,position =(10 ,0 ))
    charlie =QuantumNode (name ="Charlie",num_qubits =3 ,position =(5 ,10 ))

    network .add_node (alice )
    network .add_node (bob )
    network .add_node (charlie )

    print ("\n🌐 Network created:")
    print (f"   - 3 nós: {[n .name for n in network .nodes ]}")
    print (f"   - Topologia: Triângulo")

    # Create links (canais EFAL)
    print ("\n🔗 Creating quantum channels (EFAL)...")

    link_ab =network .create_link (alice ,bob ,fidelity =0.95 ,distance_km =10 )
    link_bc =network .create_link (bob ,charlie ,fidelity =0.92 ,distance_km =12 )
    link_ca =network .create_link (charlie ,alice ,fidelity =0.93 ,distance_km =11 )

    print (f"   Alice ↔ Bob: {link_ab .fidelity *100 :.1f}% fidelidade")
    print (f"   Bob ↔ Charlie: {link_bc .fidelity *100 :.1f}% fidelidade")
    print (f"   Charlie ↔ Alice: {link_ca .fidelity *100 :.1f}% fidelidade")

    # Distribuir pairs de Bell (protocol EMF)
    print ("\n📦 Distributing EPR pairs via EMF...")

    protocol =BellPairDistribution (network )
    pairs_distributed =protocol .distribute_pairs (
    links =[link_ab ,link_bc ,link_ca ],
    pairs_per_link =5 ,
    use_emf_pool =True 
    )

    print (f"   Pares distribuídos: {len (pairs_distributed )}")

    for pair in pairs_distributed :
        print (f"   - {pair ['link']}: fidelidade {pair ['fidelity']*100 :.2f}%")

        # Statistics EMF
    emf_stats =network .emf .get_statistics ()
    print (f"\n📊 Estatísticas EMF:")
    print (f"   Pool size: {emf_stats ['pool_size']}")
    print (f"   Pares disponíveis: {emf_stats ['available_pairs']}")
    print (f"   Fidelidade média: {emf_stats ['avg_fidelity']*100 :.2f}%")
    print (f"   Ergotropia total: {emf_stats ['total_ergotropy']:.4f}")

else :
    print ("\n⚠️  Demo mode")


    # ============================================================================
    # EXAMPLE 2: Quantum Teleportation Alice → Bob
    # ============================================================================

print ("\n"+"="*80 )
print ("EXAMPLE 2: Quantum Teleportation Alice → Bob")
print ("="*80 )

if LIGHTQOS_AVAILABLE :
# State to teleport (in Alice)
    state_to_teleport =QuantumCircuit (1 ,name ="state")
    state_to_teleport .ry (np .pi /4 ,0 )# |ψ⟩ = cos(π/8)|0⟩ + sin(π/8)|1⟩

    print ("\n📤 Alice prepared state |ψ⟩ for teleportation")
    print (f"   |ψ⟩ = cos(π/8)|0⟩ + sin(π/8)|1⟩")

    # Protocol de teleportation
    print ("\n🔄 Starting teleportation protocol...")

    teleport =Teleportation (network )
    result =teleport .execute (
    sender =alice ,
    receiver =bob ,
    state =state_to_teleport ,
    use_pre_shared_epr =True # Usar par EPR do pool EMF
    )

    # Analysis
    print (f"\n📊 Resultado:")
    print (f"   Teleportação bem-sucedida: {'✅'if result ['success']else '❌'}")
    print (f"   Fidelidade: {result ['fidelity']*100 :.2f}%")
    print (f"   Bits clássicos enviados: {result ['classical_bits']}")

    # Detalhes do protocol
    print (f"\n🔍 Detalhes do Protocolo:")
    steps =result .get ('protocol_steps',[])
    for i ,step in enumerate (steps ,1 ):
        print (f"   {i }. {step ['description']} ({step ['duration_ns']}ns)")

        # Verificar com Bob
    bob_state =result .get ('received_state')
    print (f"\n📬 Bob recebeu:")
    print (f"   Estado reconstruído: |ψ'⟩")
    print (f"   Fidelidade com original: {result ['fidelity']*100 :.2f}%")

else :
    print ("\n⚠️  Demo mode")


    # ============================================================================
    # EXEMPLO 3: Routing Inteligente com EMF
    # ============================================================================

print ("\n"+"="*80 )
print ("EXAMPLE 3: Intelligent Entanglement Routing")
print ("="*80 )

if LIGHTQOS_AVAILABLE :
# Alice quer if comunicar com Charlie (not têm link direto ativo)
    print ("\n🎯 Objective: Establish entanglement Alice ↔ Charlie")
    print ("   (without an active direct link)")

    # EMF calculates melhor route
    router =network .emf .get_router ()
    route =router .find_best_path (
    source =alice ,
    destination =charlie ,
    optimize_for ='fidelity'# ou 'latency', 'cost'
    )

    print (f"\n🗺️  Rota calculada pelo EMF:")
    print (f"   Caminho: {' → '.join ([n .name for n in route ['path']])}")
    print (f"   Distância total: {route ['total_distance']}km")
    print (f"   Fidelidade esperada: {route ['expected_fidelity']*100 :.2f}%")
    print (f"   Latência esperada: {route ['expected_latency_ms']:.2f}ms")

    # Swapping de entanglement (via Bob)
    print (f"\n🔀 Realizando Entanglement Swapping via {route ['swap_nodes'][0 ].name }...")

    swap_protocol =network .protocols ['entanglement_swapping']
    swap_result =swap_protocol .execute (
    path =route ['path'],
    use_emf =True 
    )

    print (f"\n📊 Resultado do Swapping:")
    print (f"   Entrelaçamento estabelecido: {'✅'if swap_result ['success']else '❌'}")
    print (f"   Fidelidade final: {swap_result ['final_fidelity']*100 :.2f}%")
    print (f"   Swaps realizados: {swap_result ['num_swaps']}")

    # Verificar conectividade
    connectivity =network .get_connectivity_matrix ()
    print (f"\n🌐 Matriz de Conectividade da Rede:")
    print (connectivity )

else :
    print ("\n⚠️  Demo mode")


    # ============================================================================
    # EXEMPLO 4: Distribution de Key Quantum (QKD)
    # ============================================================================

print ("\n"+"="*80 )
print ("EXAMPLE 4: Quantum Key Distribution (BB84)")
print ("="*80 )

if LIGHTQOS_AVAILABLE :
    from lightqos .protocols import BB84Protocol 

    print ("\n🔐 Protocol BB84 para distribution de key secreta")
    print ("   Alice ↔ Bob")

    # Inicializar protocol
    bb84 =BB84Protocol (network )

    # Pairameters
    key_length =256 # bits

    print (f"\n⚙️  Parâmetros:")
    print (f"   Comprimento da chave: {key_length } bits")
    print (f"   Canal: Alice ↔ Bob")
    print (f"   Fidelidade do canal: {link_ab .fidelity *100 :.1f}%")

    # Executar BB84
    print (f"\n🚀 Executando BB84...")
    qkd_result =bb84 .distribute_key (
    sender =alice ,
    receiver =bob ,
    key_length =key_length ,
    use_efal_routing =True 
    )

    # Analysis
    print (f"\n📊 Resultado:")
    print (f"   Chave gerada: ✅")
    print (f"   Comprimento final: {len (qkd_result ['key'])} bits")
    print (f"   Taxa de erro quântico (QBER): {qkd_result ['qber']*100 :.2f}%")
    print (f"   Segurança: {'✅ Secure'if qkd_result ['secure']else '⚠️ Insecure'}")

    # Statistics
    stats =qkd_result ['statistics']
    print (f"\n📈 Estatísticas:")
    print (f"   Qubits enviados: {stats ['qubits_sent']}")
    print (f"   Qubits medidos: {stats ['qubits_measured']}")
    print (f"   Taxa de sifting: {stats ['sifting_rate']*100 :.1f}%")
    print (f"   Bits pós-correção de erros: {stats ['bits_after_ec']}")
    print (f"   Bits pós-amplificação de privacidade: {stats ['bits_after_pa']}")

    # Key (first 64 bits)
    key_hex =''.join ([str (b )for b in qkd_result ['key'][:64 ]])
    print (f"\n🔑 Chave (primeiros 64 bits):")
    print (f"   {key_hex }")

else :
    print ("\n⚠️  Demo mode")


    # ============================================================================
    # EXEMPLO 5: Network com Noise e Recovery Automática (EMF + TLM)
    # ============================================================================

print ("\n"+"="*80 )
print ("EXAMPLE 5: Automatic Recovery in a Noisy Network")
print ("="*80 )

if LIGHTQOS_AVAILABLE :
    print ("\n⚠️  Simulating link failure Bob ↔ Charlie...")

    # Simular failure
    link_bc .inject_noise (error_rate =0.3 )# 30% error

    print (f"   Link degradado: fidelidade {link_bc .fidelity *100 :.1f}% → "
    f"{link_bc .get_current_fidelity ()*100 :.1f}%")

    # EMF detecta e reage
    print (f"\n🔧 EMF detectando degradação...")

    emf_action =network .emf .handle_link_degradation (link_bc )

    print (f"\n📊 Ação do EMF:")
    print (f"   - {emf_action ['action']}")

    if emf_action ['action']=='reroute':
        print (f"   - Nova rota: {' → '.join ([n .name for n in emf_action ['new_route']])}")
        print (f"   - Fidelidade esperada: {emf_action ['new_fidelity']*100 :.1f}%")

    elif emf_action ['action']=='purification':
        print (f"   - Purificação ativada")
        print (f"   - Pares usados para purificação: {emf_action ['pairs_used']}")
        print (f"   - Fidelidade pós-purificação: {emf_action ['purified_fidelity']*100 :.1f}%")

        # TLM ajusta timing
    print (f"\n⏱️  TLM ajustando contratos temporais...")

    tlm_adjustments =network .tlm .adjust_for_degradation (link_bc )

    print (f"   Novos limites temporais:")
    for node_name ,contract in tlm_adjustments .items ():
        print (f"   - {node_name }: {contract ['max_duration_ns']}ns "
        f"(tolerância: ±{contract ['tolerance_ns']}ns)")

        # Statistics de recovery
    recovery_stats =network .get_recovery_statistics ()
    print (f"\n📈 Estatísticas de Recuperação:")
    print (f"   Falhas detectadas: {recovery_stats ['failures_detected']}")
    print (f"   Recuperações automáticas: {recovery_stats ['auto_recoveries']}")
    print (f"   Taxa de sucesso: {recovery_stats ['success_rate']*100 :.1f}%")
    print (f"   Tempo médio de recuperação: {recovery_stats ['avg_recovery_time_ms']:.2f}ms")

else :
    print ("\n⚠️  Demo mode")


    # ============================================================================
    # EXEMPLO 6: Network de 10 Nodes in Topologia de Malha
    # ============================================================================

print ("\n"+"="*80 )
print ("EXEMPLO 6: Network de Grande Escala (10 Nodes)")
print ("="*80 )

if LIGHTQOS_AVAILABLE :
# Create network grande
    large_net =QuantumNetwork (name ="qnet_large",topology ='mesh')

    print ("\n🌐 Creating network de 10 nodes in topologia de malha...")

    # Add nodes
    nodes =[]
    for i in range (10 ):
        node =QuantumNode (
        name =f"Node_{i }",
        num_qubits =5 ,
        position =(np .random .rand ()*100 ,np .random .rand ()*100 )
        )
        large_net .add_node (node )
        nodes .append (node )

        # Create malha (cada node conectado aos 3 mais próximos)
    print (f"\n🔗 Criando links (cada nó → 3 vizinhos mais próximos)...")

    num_links =large_net .create_mesh_topology (
    nodes_per_connection =3 ,
    min_fidelity =0.85 
    )

    print (f"   Links criados: {num_links }")

    # Statistics da network
    net_stats =large_net .get_statistics ()
    print (f"\n📊 Estatísticas da Rede:")
    print (f"   Nós: {net_stats ['num_nodes']}")
    print (f"   Links: {net_stats ['num_links']}")
    print (f"   Grau médio: {net_stats ['avg_degree']:.1f}")
    print (f"   Diâmetro da rede: {net_stats ['diameter']} hops")
    print (f"   Fidelidade média: {net_stats ['avg_fidelity']*100 :.1f}%")

    # Teste de throughput
    print (f"\n🚀 Testando throughput da rede...")

    throughput_test =large_net .benchmark_throughput (
    duration_seconds =10 ,
    protocol ='epr_distribution'
    )

    print (f"\n📈 Resultado do Benchmark:")
    print (f"   Pares EPR/segundo: {throughput_test ['eprs_per_second']:.0f}")
    print (f"   Largura de banda: {throughput_test ['bandwidth_qubits_per_sec']:.0f} qubits/s")
    print (f"   Latência média: {throughput_test ['avg_latency_ms']:.2f}ms")
    print (f"   Utilização média EMF: {throughput_test ['avg_emf_utilization']*100 :.1f}%")

else :
    print ("\n⚠️  Demo mode")


    # ============================================================================
    # RESUMO
    # ============================================================================

print ("\n"+"="*80 )
print ("RESUMO: Capacidades de Network Quantum LightQOS")
print ("="*80 )

print ("""
✅ Topologias Suportadas:
   - Ponto-a-ponto
   - Triângulo / Estrela
   - Malha (Mesh)
   - Personalizada

✅ Protocols Implementados:
   - Distribution EPR (Bell pairs)
   - Teleportation Quantum
   - Entanglement Swapping
   - QKD (BB84, E91)
   - Purification de Entanglement

✅ Funcionalidades EFAL:
   - Routing dinâmico de canais
   - Optimization de topologia
   - Gestão de largura de banda quantum

✅ Funcionalidades EMF:
   - Pool de entanglement global
   - Routing inteligente
   - Recovery automática
   - Métricas termodinâmicas

✅ Funcionalidades TLM:
   - Sincronização de network
   - Contracts temporal distributed
   - Ajuste adaptativo de timing

✅ Escalabilidade:
   - Tstate até 100 nodes
   - Suporta malhas arbitrárias
   - Throughput optimized

📚 Next exemplos:
   - quantum_internet.py (Internet Quantum global)
   - secure_voting.py (Votação secure com QKD)
   - distributed_computation.py (Computação distribuída)
""")

print ("\n✅ Example complete! Network quantum funcional.")
