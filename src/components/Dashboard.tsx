import React, { useState } from 'react';
import { useMetasys } from '../hooks/useMetasys';
import type { UmaPoint } from '../types';
import styles from './Dashboard.module.css';

const PointRow = ({ umaName, point, onCommand }: { umaName: string, point: UmaPoint, onCommand: (umaName: string, pointName: string, ref: string, action: string, label: string) => void }) => {
  const isTemp = point.control_type === 'temperature';
  const initialVal = isTemp ? 20 : 50;
  const [analogValue, setAnalogValue] = useState(initialVal);

  if (point.control_type === 'analog' || point.control_type === 'temperature') {
    const minVal = isTemp ? 15 : 0;
    const maxVal = isTemp ? 30 : 100;
    const unit = isTemp ? '°C' : '%';
    return (
      <div className={styles.umaRow}>
        <div className={styles.umaName}>{point.name}</div>
        <div className={styles.umaControls}>
          <input 
            type="range" 
            min={minVal} 
            max={maxVal} 
            value={analogValue} 
            onChange={(e) => setAnalogValue(Number(e.target.value))}
            className={styles.sleekSlider}
          />
          <span className={styles.analogValue}>{analogValue}{unit}</span>
          <button 
            className={styles.btnClean}
            onClick={() => onCommand(umaName, point.name, point.reference, analogValue.toString(), `SOBRECOMANDAR A ${analogValue}${unit}`)}
          >
            Fijar
          </button>
          <button 
            className={styles.btnAuto}
            onClick={() => onCommand(umaName, point.name, point.reference, 'RELEASE', 'LIBERAR')}
          >
            Auto
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.umaRow}>
      <div className={styles.umaName}>{point.name}</div>
      <div className={styles.segmentedControl}>
        <button 
          className={styles.segBtn}
          onClick={() => onCommand(umaName, point.name, point.reference, '1', 'SOBRECOMANDAR (ON)')}
        >
          ON
        </button>
        <button 
          className={styles.segBtn}
          onClick={() => onCommand(umaName, point.name, point.reference, '0', 'SOBRECOMANDAR (OFF)')}
        >
          OFF
        </button>
        <button 
          className={`${styles.segBtn} ${styles.segBtnAuto}`}
          onClick={() => onCommand(umaName, point.name, point.reference, 'RELEASE', 'LIBERAR')}
        >
          AUTO
        </button>
      </div>
    </div>
  );
};

const EquipmentCard = ({ uma, onCommand, onEnableSchedule }: { uma: any, onCommand: any, onEnableSchedule?: () => void }) => {
  const [expanded, setExpanded] = useState(false);

  // Buscar el punto de "Habilitación UMA"
  const habilitacionPoint = uma.points.find((p: any) => p.name.toLowerCase().includes("habilitaci"));
  
  // Todos los demás puntos, excluyendo el ventilador (que está oculto) y la habilitación
  const otherPoints = uma.points.filter((p: any) => 
    p.name !== "Comando Ventilador" && p.id !== habilitacionPoint?.id
  );

  return (
    <div className={styles.equipmentCard}>
      <div 
        className={styles.equipmentHeader} 
        onClick={() => setExpanded(!expanded)}
        style={{ cursor: 'pointer', borderBottom: (expanded || habilitacionPoint) ? '1px solid var(--border-subtle)' : 'none', paddingBottom: (expanded || habilitacionPoint) ? '1rem' : '0', marginBottom: (expanded || habilitacionPoint) ? '1rem' : '0' }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
          <h3 className={styles.equipmentTitle}>{uma.name}</h3>
          {onEnableSchedule && (
            <button 
              className={styles.segBtn} 
              style={{ background: 'var(--primary)', color: 'var(--bg-card)', padding: '0.25rem 0.75rem', fontSize: '0.85rem' }}
              onClick={(e) => { e.stopPropagation(); onEnableSchedule(); }}
            >
              Habilitar Horario
            </button>
          )}
        </div>
        <button 
          className={styles.expandBtn}
          style={{ transform: expanded ? 'rotate(180deg)' : 'rotate(0)' }}
        >
          ▼
        </button>
      </div>
      
      {/* Siempre mostramos la habilitación de UMA (si existe) */}
      {habilitacionPoint && (
        <div style={{ marginBottom: expanded ? '1rem' : '0' }}>
          <PointRow umaName={uma.name} point={habilitacionPoint} onCommand={onCommand} />
        </div>
      )}

      {/* Solo mostramos el resto si está expandido */}
      {expanded && (
        <div className={styles.umasList}>
          {otherPoints.length === 0 ? (
            <p style={{ fontSize: '0.8rem', color: 'var(--text-secondary)' }}>Sin puntos adicionales.</p>
          ) : (
            otherPoints.map((point: any) => (
              <PointRow key={point.id} umaName={uma.name} point={point} onCommand={onCommand} />
            ))
          )}
        </div>
      )}
    </div>
  );
};

const Dashboard: React.FC = () => {
  const { umas, queue, enqueueCommand, invokeSchedule } = useMetasys();
  const [filterText, setFilterText] = useState('');

  const handleCommand = (umaName: string, pointName: string, reference: string, action: string, actionName: string) => {
    if (window.confirm(`¿Estás seguro de que deseas ${actionName} en:\n${umaName} -> ${pointName}?`)) {
      enqueueCommand(umaName, pointName, reference, action);
      
      // Macro oculta: Si mandamos Habilitación UMA, replicar el mismo comando al Ventilador
      if (pointName === "Habilitación UMA") {
        const uma = umas.find(u => u.name === umaName);
        if (uma) {
          const vent = uma.points.find(p => p.name === "Comando Ventilador");
          if (vent) {
            enqueueCommand(umaName, vent.name, vent.reference, action);
          }
        }
      }
    }
  };

  const handleEnableSchedule = async () => {
    if (window.confirm("¿Estás seguro de que deseas habilitar el horario de todas las luces del Piso 1?")) {
      const payload = {
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.50% Norte Hall espera CAE Ponien": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.Espera Adultos": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.Espera farmacia": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.Espera Infantil": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.Focos Exterior Fachada 18 septie": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.Hall Acceso CAE": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.Hall ascensores TN2C Farmacia": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.Hall espera CAE oriente": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-EXTERIOR.Horario Exteriores": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.Letrero Exterior CAE": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.50% sur hall espera CAE Poniente": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.Pasillo circulacion Box especial": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.Pasillo circulacion DAN": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.Pasillo  Interior Recepcion CAE": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.Pasillo circulacion sur estacion": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.Pasillo especialida sur y norte": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.Postes Alumbrado Estacionamiento": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO-ALUM-CAE.PISO 1.Rampa acceso CAE": true,
        "HDA-Metasys10:NAE-HArica/Schedule.LUCES.CTO- ALUM-ZOCALO.ZOCALO ASCENSORES  TRIPLE": true
      };
      
      try {
        await invokeSchedule(payload);
        alert("Horarios habilitados exitosamente en Metasys.");
      } catch (e: any) {
        alert("Error al habilitar horarios: " + e);
      }
    }
  };

  const getBadgeClass = (status: string) => {
    switch (status) {
      case 'PENDING': return styles.pending;
      case 'PROCESSING': return styles.processing;
      case 'RETRYING': return styles.retrying;
      case 'COMPLETED': return styles.completed;
      case 'FAILED': return styles.failed;
      default: return '';
    }
  };

  const getStatusLabel = (status: string) => {
    switch (status) {
      case 'PENDING': return 'En Cola';
      case 'PROCESSING': return 'Enviando...';
      case 'RETRYING': return 'Reintentando';
      case 'COMPLETED': return 'Exitoso';
      case 'FAILED': return 'Fallido';
      default: return status;
    }
  };

  const filteredUmas = umas.filter(uma => uma.name.toLowerCase().includes(filterText.toLowerCase()));

  return (
    <div className={styles.grid}>
      <section className={styles.card}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '2rem' }}>
          <h2 className={styles.cardTitle} style={{ marginBottom: 0 }}>Equipos y Puntos de Control</h2>
          <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
            <input 
              type="text" 
              placeholder="Buscar equipo..." 
              value={filterText}
              onChange={(e) => setFilterText(e.target.value)}
              className={styles.filterInput}
            />
          </div>
        </div>
        <div className={styles.umasListContainer}>
          {filteredUmas.length === 0 ? (
            <p style={{ color: 'var(--text-secondary)' }}>No se encontraron equipos.</p>
          ) : (
            filteredUmas.map((uma) => (
              <EquipmentCard 
                key={uma.id} 
                uma={uma} 
                onCommand={handleCommand}
                onEnableSchedule={uma.name.includes("Luces Edificio CAE") ? handleEnableSchedule : undefined}
              />
            ))
          )}
        </div>
      </section>

      <section className={styles.card}>
        <h2 className={styles.cardTitle}>Cola de Comandos</h2>
        <div className={styles.queueList}>
          {queue.length === 0 ? (
            <p style={{ color: 'var(--text-secondary)', fontSize: '0.875rem' }}>La cola está vacía.</p>
          ) : (
            queue.slice().reverse().map((cmd) => {
              // Try to figure out if it was analog or toggle for display
              let displayAction = cmd.action;
              if (cmd.action === '1') displayAction = 'ON';
              else if (cmd.action === '0') displayAction = 'OFF';
              else if (cmd.action !== 'RELEASE') {
                // Determine if it's temperature or analog based on value/uma configuration (we don't have direct access here easily, so we just append the action string directly if it's not a toggle)
                // Actually, the command action is the raw string. We'll just show the raw string.
                displayAction = cmd.action;
              }

              return (
                <div key={cmd.id} className={styles.queueItem}>
                  <div className={styles.queueItemInfo}>
                    <div className={styles.queueItemTitle}>
                      {cmd.uma_name}
                      {cmd.retries > 0 && <span style={{ color: 'var(--warning)', marginLeft: '0.5rem', fontSize: '0.75rem' }}>({cmd.retries} reint.)</span>}
                    </div>
                    <div className={styles.queueItemSub}>
                      {cmd.point_name} <span className={styles.queueItemAction}>[{displayAction}]</span>
                    </div>
                  </div>
                  <span className={`${styles.badge} ${getBadgeClass(cmd.status)}`}>
                    {getStatusLabel(cmd.status)}
                  </span>
                </div>
              );
            })
          )}
        </div>
      </section>
    </div>
  );
};

export default Dashboard;
