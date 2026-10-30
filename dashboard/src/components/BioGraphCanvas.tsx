import React, { useRef, useEffect, useState, useCallback } from 'react';
import { EcmaNodeData, GraphEdgeData } from '../types/engine';

interface BioGraphCanvasProps {
  nodes: EcmaNodeData[];
  edges: GraphEdgeData[];
  selectedNode: EcmaNodeData | null;
  onSelectNode: (node: EcmaNodeData | null) => void;
  showPheromoneHighways: boolean;
  activePathNodes: number[];
}

export const BioGraphCanvas: React.FC<BioGraphCanvasProps> = ({
  nodes,
  edges,
  selectedNode,
  onSelectNode,
  showPheromoneHighways,
  activePathNodes,
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  // Transform controls
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const isDraggingCanvas = useRef(false);
  const dragStart = useRef({ x: 0, y: 0 });
  const draggedNode = useRef<EcmaNodeData | null>(null);

  // Animation frame ref
  const animFrameId = useRef<number | null>(null);

  // Colors per node state
  const getStateColors = (state: EcmaNodeData['state'], isPathogen?: boolean) => {
    if (isPathogen) {
      return { core: '#ef4444', ring: 'rgba(239, 68, 68, 0.7)', glow: 'rgba(239, 68, 68, 0.4)' };
    }
    switch (state) {
      case 'Stem':
        return { core: '#10b981', ring: 'rgba(16, 185, 129, 0.7)', glow: 'rgba(16, 185, 129, 0.35)' };
      case 'Progenitor':
        return { core: '#06b6d4', ring: 'rgba(6, 182, 212, 0.7)', glow: 'rgba(6, 182, 212, 0.35)' };
      case 'Specialized':
        return { core: '#a855f7', ring: 'rgba(168, 85, 247, 0.8)', glow: 'rgba(168, 85, 247, 0.45)' };
      case 'Apoptosis':
        return { core: '#ef4444', ring: 'rgba(239, 68, 68, 0.4)', glow: 'rgba(239, 68, 68, 0.2)' };
    }
  };

  // Center nodes initially if canvas is mounted
  useEffect(() => {
    const canvas = canvasRef.current;
    if (canvas) {
      setPan({ x: canvas.width / 2, y: canvas.height / 2 });
    }
  }, []);

  // Main physics & rendering animation loop
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let pulseTime = 0;

    const render = () => {
      pulseTime += 0.035;

      // Ensure canvas sizing matches client bounding rect
      const dpr = window.devicePixelRatio || 1;
      const width = canvas.clientWidth;
      const height = canvas.clientHeight;
      if (canvas.width !== width * dpr || canvas.height !== height * dpr) {
        canvas.width = width * dpr;
        canvas.height = height * dpr;
      }

      ctx.save();
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, width, height);

      // Draw subtle background bio-grid
      ctx.save();
      ctx.translate(pan.x, pan.y);
      ctx.scale(zoom, zoom);

      ctx.strokeStyle = 'rgba(31, 37, 56, 0.35)';
      ctx.lineWidth = 1 / zoom;
      const gridSize = 80;
      const startX = -1200;
      const endX = 1200;
      const startY = -1000;
      const endY = 1000;

      for (let x = startX; x <= endX; x += gridSize) {
        ctx.beginPath();
        ctx.moveTo(x, startY);
        ctx.lineTo(x, endY);
        ctx.stroke();
      }
      for (let y = startY; y <= endY; y += gridSize) {
        ctx.beginPath();
        ctx.moveTo(startX, y);
        ctx.lineTo(endX, y);
        ctx.stroke();
      }

      // Physics update: simple spring & repulsion layout
      for (let i = 0; i < nodes.length; i++) {
        const n1 = nodes[i];
        if (n1 === draggedNode.current) continue;

        // Repulsion between nodes
        for (let j = i + 1; j < nodes.length; j++) {
          const n2 = nodes[j];
          const dx = (n1.x || 0) - (n2.x || 0);
          const dy = (n1.y || 0) - (n2.y || 0);
          const distSq = dx * dx + dy * dy + 10;
          const dist = Math.sqrt(distSq);

          if (dist < 260) {
            const force = 350 / distSq;
            const fx = (dx / dist) * force;
            const fy = (dy / dist) * force;

            if (n1 !== draggedNode.current) {
              n1.vx = (n1.vx || 0) + fx;
              n1.vy = (n1.vy || 0) + fy;
            }
            if (n2 !== draggedNode.current) {
              n2.vx = (n2.vx || 0) - fx;
              n2.vy = (n2.vy || 0) - fy;
            }
          }
        }

        // Center gravity pulling towards (0,0)
        n1.vx = (n1.vx || 0) - (n1.x || 0) * 0.002;
        n1.vy = (n1.vy || 0) - (n1.y || 0) * 0.002;

        // Velocity damping
        n1.vx = (n1.vx || 0) * 0.88;
        n1.vy = (n1.vy || 0) * 0.88;

        n1.x = (n1.x || 0) + (n1.vx || 0);
        n1.y = (n1.y || 0) + (n1.vy || 0);
      }

      // Spring forces from edges
      for (const edge of edges) {
        const n1 = nodes.find(n => n.id === edge.from);
        const n2 = nodes.find(n => n.id === edge.to);
        if (!n1 || !n2) continue;

        const dx = (n2.x || 0) - (n1.x || 0);
        const dy = (n2.y || 0) - (n1.y || 0);
        const dist = Math.sqrt(dx * dx + dy * dy) || 1;
        const targetDist = 140 - Math.min(edge.pheromone, 8) * 8;
        const springForce = (dist - targetDist) * 0.004;

        const fx = (dx / dist) * springForce;
        const fy = (dy / dist) * springForce;

        if (n1 !== draggedNode.current) {
          n1.vx = (n1.vx || 0) + fx;
          n1.vy = (n1.vy || 0) + fy;
        }
        if (n2 !== draggedNode.current) {
          n2.vx = (n2.vx || 0) - fx;
          n2.vy = (n2.vy || 0) - fy;
        }
      }

      // Draw Edges (Pheromone Trails)
      for (const edge of edges) {
        const n1 = nodes.find(n => n.id === edge.from);
        const n2 = nodes.find(n => n.id === edge.to);
        if (!n1 || !n2) continue;

        const x1 = n1.x || 0;
        const y1 = n1.y || 0;
        const x2 = n2.x || 0;
        const y2 = n2.y || 0;

        const isHighway = edge.pheromone > 3.0 && showPheromoneHighways;
        const isPathActive = activePathNodes.includes(edge.from) && activePathNodes.includes(edge.to);

        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);

        if (isPathActive) {
          ctx.strokeStyle = '#38bdf8';
          ctx.lineWidth = 3.5;
          ctx.shadowColor = '#38bdf8';
          ctx.shadowBlur = 12;
        } else if (isHighway) {
          // Bioluminescent golden highway
          ctx.strokeStyle = `rgba(245, 158, 11, ${Math.min(0.3 + edge.pheromone * 0.1, 0.95)})`;
          ctx.lineWidth = Math.min(1.5 + edge.pheromone * 0.6, 5);
          ctx.shadowColor = 'rgba(245, 158, 11, 0.8)';
          ctx.shadowBlur = 8;
        } else {
          // Normal synapse edge
          ctx.strokeStyle = 'rgba(75, 85, 99, 0.35)';
          ctx.lineWidth = 1;
          ctx.shadowBlur = 0;
        }
        ctx.stroke();
        ctx.shadowBlur = 0;

        // Draw traveling particles (Ants / Context Packets) along highway edges
        if ((isHighway || isPathActive) && edge.particles) {
          for (const p of edge.particles) {
            p.progress += p.speed;
            if (p.progress > 1) p.progress = 0;

            const px = x1 + (x2 - x1) * p.progress;
            const py = y1 + (y2 - y1) * p.progress;

            ctx.beginPath();
            ctx.arc(px, py, isPathActive ? 3.5 : 2.5, 0, Math.PI * 2);
            ctx.fillStyle = isPathActive ? '#7dd3fc' : '#fbbf24';
            ctx.shadowColor = isPathActive ? '#38bdf8' : '#f59e0b';
            ctx.shadowBlur = 6;
            ctx.fill();
            ctx.shadowBlur = 0;
          }
        }
      }

      // Draw Nodes (Biological Cells)
      for (const node of nodes) {
        const x = node.x || 0;
        const y = node.y || 0;
        const baseRadius = node.radius || 12;
        const isSelected = selectedNode?.id === node.id;
        const isActiveQuery = activePathNodes.includes(node.id);
        const { core, ring, glow } = getStateColors(node.state, node.isPathogen);

        // Pulse calculation
        const pulse = Math.sin(pulseTime + node.id) * 2;
        const effectiveRadius = baseRadius + (isSelected ? 3 : 0) + (isActiveQuery ? 4 : 0);

        // Outer glow aura
        ctx.beginPath();
        ctx.arc(x, y, effectiveRadius + 8 + pulse, 0, Math.PI * 2);
        ctx.fillStyle = glow;
        ctx.fill();

        // Membrane ring
        ctx.beginPath();
        ctx.arc(x, y, effectiveRadius + 2, 0, Math.PI * 2);
        ctx.strokeStyle = isSelected ? '#ffffff' : isActiveQuery ? '#38bdf8' : ring;
        ctx.lineWidth = isSelected ? 3 : 2;
        ctx.stroke();

        // Cell nucleus / core
        ctx.beginPath();
        ctx.arc(x, y, effectiveRadius - 2, 0, Math.PI * 2);
        ctx.fillStyle = core;
        ctx.fill();

        // Node label
        ctx.font = `${Math.max(10, 11 / zoom)}px "JetBrains Mono", monospace`;
        ctx.fillStyle = isSelected ? '#ffffff' : 'rgba(241, 245, 249, 0.85)';
        ctx.textAlign = 'center';
        ctx.fillText(node.label || `N:${node.id}`, x, y + effectiveRadius + 14);

        // State mini-badge
        ctx.font = '8px sans-serif';
        ctx.fillStyle = 'rgba(148, 163, 184, 0.7)';
        ctx.fillText(node.state.toUpperCase(), x, y + effectiveRadius + 24);
      }

      ctx.restore();
      ctx.restore();

      animFrameId.current = requestAnimationFrame(render);
    };

    animFrameId.current = requestAnimationFrame(render);

    return () => {
      if (animFrameId.current) {
        cancelAnimationFrame(animFrameId.current);
      }
    };
  }, [nodes, edges, selectedNode, pan, zoom, showPheromoneHighways, activePathNodes]);

  // Handle zoom
  const handleWheel = (e: React.WheelEvent) => {
    e.preventDefault();
    const zoomFactor = e.deltaY < 0 ? 1.1 : 0.9;
    setZoom(z => Math.min(Math.max(z * zoomFactor, 0.3), 3.0));
  };

  // Convert client coordinate to graph coordinate
  const clientToGraph = useCallback(
    (clientX: number, clientY: number) => {
      const canvas = canvasRef.current;
      if (!canvas) return { x: 0, y: 0 };
      const rect = canvas.getBoundingClientRect();
      const rawX = clientX - rect.left;
      const rawY = clientY - rect.top;
      return {
        x: (rawX - pan.x) / zoom,
        y: (rawY - pan.y) / zoom,
      };
    },
    [pan, zoom]
  );

  // Mouse handlers for dragging nodes and panning canvas
  const handleMouseDown = (e: React.MouseEvent) => {
    const pos = clientToGraph(e.clientX, e.clientY);

    // Check if clicked on a node
    let clicked: EcmaNodeData | null = null;
    for (const node of nodes) {
      const dx = (node.x || 0) - pos.x;
      const dy = (node.y || 0) - pos.y;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist <= (node.radius || 12) + 6) {
        clicked = node;
        break;
      }
    }

    if (clicked) {
      draggedNode.current = clicked;
      onSelectNode(clicked);
    } else {
      isDraggingCanvas.current = true;
      dragStart.current = { x: e.clientX - pan.x, y: e.clientY - pan.y };
      onSelectNode(null);
    }
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (draggedNode.current) {
      const pos = clientToGraph(e.clientX, e.clientY);
      draggedNode.current.x = pos.x;
      draggedNode.current.y = pos.y;
      draggedNode.current.vx = 0;
      draggedNode.current.vy = 0;
    } else if (isDraggingCanvas.current) {
      setPan({
        x: e.clientX - dragStart.current.x,
        y: e.clientY - dragStart.current.y,
      });
    }
  };

  const handleMouseUp = () => {
    draggedNode.current = null;
    isDraggingCanvas.current = false;
  };

  return (
    <div className="relative w-full h-full select-none overflow-hidden bg-bio-bg">
      <canvas
        ref={canvasRef}
        onWheel={handleWheel}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
        className="w-full h-full cursor-grab active:cursor-grabbing block"
      />

      {/* Floating Canvas Controls */}
      <div className="absolute bottom-5 left-5 flex items-center gap-2 bg-bio-surface/90 backdrop-blur border border-bio-border px-3 py-2 rounded-xl text-xs text-bio-muted shadow-2xl">
        <button
          onClick={() => setZoom(z => Math.min(z * 1.2, 3.0))}
          className="px-2 py-1 bg-bio-card hover:bg-bio-border text-bio-text rounded transition"
          title="Zoom in"
        >
          +
        </button>
        <span className="font-mono min-w-[42px] text-center">{Math.round(zoom * 100)}%</span>
        <button
          onClick={() => setZoom(z => Math.max(z * 0.8, 0.3))}
          className="px-2 py-1 bg-bio-card hover:bg-bio-border text-bio-text rounded transition"
          title="Zoom out"
        >
          -
        </button>
        <div className="w-[1px] h-4 bg-bio-border mx-1" />
        <button
          onClick={() => {
            const canvas = canvasRef.current;
            if (canvas) setPan({ x: canvas.clientWidth / 2, y: canvas.clientHeight / 2 });
            setZoom(1);
          }}
          className="px-2 py-1 bg-bio-card hover:bg-bio-border text-bio-text rounded transition"
          title="Reset position"
        >
          Reset View
        </button>
      </div>

      {/* Biological Legend */}
      <div className="absolute top-5 left-5 bg-bio-surface/90 backdrop-blur border border-bio-border p-3 rounded-xl shadow-2xl flex flex-col gap-2 text-xs">
        <div className="text-[11px] font-semibold tracking-wider text-bio-muted uppercase">ECMA Lifecycle</div>
        <div className="flex items-center gap-2">
          <span className="w-3 h-3 rounded-full bg-bio-stem glow-stem inline-block" />
          <span className="text-bio-text">Stem (Cell Embryo)</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="w-3 h-3 rounded-full bg-bio-progenitor inline-block" />
          <span className="text-bio-text">Progenitor (Growing)</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="w-3 h-3 rounded-full bg-bio-specialized glow-specialized inline-block" />
          <span className="text-bio-text">Specialized (Mature Core)</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="w-3 h-3 rounded-full bg-bio-apoptosis glow-apoptosis inline-block" />
          <span className="text-bio-text">Apoptosis (Decaying)</span>
        </div>
        <div className="w-full h-[1px] bg-bio-border my-1" />
        <div className="flex items-center gap-2">
          <span className="w-4 h-1 rounded bg-bio-pheromone glow-pheromone inline-block" />
          <span className="text-bio-pheromone font-medium">ACO Pheromone Highway</span>
        </div>
      </div>
    </div>
  );
};
