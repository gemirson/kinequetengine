# 🧬 KineStudio — Living Cognitive Console for KineQuetEngine

O **KineStudio** é o Web Dashboard e Console Vivo projetado especificamente para o **KineQuetEngine (KCE)**. Ao contrário de interfaces para bancos de dados passivos, o KineStudio visualiza o estado de um organismo cognitivo bio-inspirado em tempo real.

---

## 🌟 Funcionalidades Principais

### 1. 🌌 Grafo Vivo Biológico (BioGraphCanvas)
* **Ciclo Celular ECMA**:
  * 🟢 **Stem (Célula-Tronco)**: Nós embrionários de conhecimento recém-adquirido.
  * 🔵 **Progenitor**: Nós em diferenciação e expansão de conexões.
  * 🟣 **Specialized (Madura)**: Nós de alta densidade e maturidade no núcleo cognitivo.
  * 🔴 **Apoptosis**: Conhecimento obsoleto marcado para reciclagem e morte celular.
* **Rodovias de Feromônios (ACO - Ant Colony Optimization)**:
  * Arestas com intensidade de feromônio ($\tau$) reforçada brilham em dourado (*Cognitive Highways*).
  * Formigas / pacotes de contexto viajam ativamente pelas rotas de menor resistência e maior relevância.
  * Física orgânica acelerada com arrastar de nós, zoom, pan e centralização.

### 2. 🔍 Node Inspector
* Ao clicar em qualquer nó do grafo, o painel lateral abre detalhando:
  * ID e Rótulo do Nó
  * **Fator de Maturidade** (0% a 100%)
  * **Nível de Entropia** (Estável vs. Caótico)
  * Histórico de ativações e contagem de sinapses
  * Sinapses vizinhas e valor $\tau$ de feromônio
  * **Intervenções Biológicas**: Induzir diferenciação, reforçar feromônios ou disparar apoptose.

### 3. 🛡️ AIS Immune Shield & Teste de Ameaça
* Botão **"Test AIS Shield"**: injeta um antígeno / *prompt injection* semântico para demonstrar o escudo do Artificial Immune System bloqueando o patógeno e aplicando apoptose punitiva imediata.

### 4. ⚡ Telemetria do Pipeline de 10 Estágios
* Visualizador colapsável que mostra a duração e métricas de cada estágio:
  1. *Validation & Rate Limiting*
  2. *AIS Threat Filter*
  3. *SIMD Vector Retrieval*
  4. *ACO Highway Routing*
  5. *Semantic Expansion*
  6. *ECMA State Decay*
  7. *mRNA Synthesis*
  8. *Action Execution*
  9. *Pheromone Evaporation*
  10. *WAL Checksum Commit*

---

## 🚀 Como Executar

```bash
# 1. Acesse o diretório do dashboard
cd dashboard

# 2. Instale as dependências (se ainda não o fez)
npm install

# 3. Inicie o servidor de desenvolvimento
npm run dev
```

Abra seu navegador em [http://localhost:3000](http://localhost:3000).

O console funciona de forma 100% autônoma no **Simulation Mode** e também tenta se conectar ao backend real do KCE em `http://localhost:8080` (proxy `/api` já configurado no Vite).
