import React, { useState } from 'react';
import { useMetasys } from '../hooks/useMetasys';
import styles from './Settings.module.css';

const Settings: React.FC = () => {
  const { umas, addUma, addPoint, removeUma, removePoint } = useMetasys();
  
  // Estado para nuevo Equipo
  const [umaName, setUmaName] = useState('');

  // Estado para nuevo Punto de Control
  const [selectedUmaId, setSelectedUmaId] = useState('');
  const [pointName, setPointName] = useState('');
  const [reference, setReference] = useState('');
  const [controlType, setControlType] = useState<'toggle' | 'analog' | 'temperature'>('toggle');

  const handleAddUma = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!umaName.trim()) return;
    await addUma(umaName);
    setUmaName('');
  };

  const handleAddPoint = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedUmaId || !pointName.trim() || !reference.trim()) return;
    await addPoint(selectedUmaId, pointName, reference, controlType);
    setPointName('');
    setReference('');
    setControlType('toggle');
  };

  return (
    <div className={styles.container}>
      <h2 className={styles.title}>1. Agregar Nuevo Equipo (UMA)</h2>
      <form className={styles.form} onSubmit={handleAddUma} style={{ marginBottom: '3rem' }}>
        <div className={styles.formGroup}>
          <label htmlFor="umaName">Nombre del Equipo</label>
          <input
            id="umaName"
            type="text"
            className={styles.input}
            value={umaName}
            onChange={(e) => setUmaName(e.target.value)}
            placeholder="Ej. Pabellón 2"
            required
          />
        </div>
        <button type="submit" className={styles.btnSubmit} disabled={!umaName.trim()}>
          Guardar Equipo
        </button>
      </form>

      <h2 className={styles.title}>2. Agregar Punto de Control a Equipo</h2>
      <form className={styles.form} onSubmit={handleAddPoint}>
        <div className={styles.formGroup}>
          <label htmlFor="selectUma">Seleccionar Equipo</label>
          <select 
            id="selectUma" 
            className={styles.input}
            value={selectedUmaId}
            onChange={(e) => setSelectedUmaId(e.target.value)}
            required
          >
            <option value="">-- Elija un Equipo --</option>
            {umas.map(u => <option key={u.id} value={u.id}>{u.name}</option>)}
          </select>
        </div>

        <div className={styles.formGroup}>
          <label htmlFor="pointName">Nombre del Punto</label>
          <input
            id="pointName"
            type="text"
            className={styles.input}
            value={pointName}
            onChange={(e) => setPointName(e.target.value)}
            placeholder="Ej. Válvula de Calor"
            required
          />
        </div>
        
        <div className={styles.formGroup}>
          <label htmlFor="reference">Reference Endpoint</label>
          <input
            id="reference"
            type="text"
            className={styles.input}
            value={reference}
            onChange={(e) => setReference(e.target.value)}
            placeholder="HDA-Metasys...HTG SL-O"
            required
          />
        </div>

        <div className={styles.formGroup}>
          <label htmlFor="controlType">Tipo de Control</label>
          <select 
            id="controlType" 
            className={styles.input}
            value={controlType}
            onChange={(e) => setControlType(e.target.value as 'toggle' | 'analog' | 'temperature')}
          >
            <option value="toggle">Interruptor (ON / OFF)</option>
            <option value="analog">Válvula Analógica (0% - 100%)</option>
            <option value="temperature">Setpoint Temperatura (°C)</option>
          </select>
        </div>

        <button 
          type="submit" 
          className={styles.btnSubmit}
          disabled={!selectedUmaId || !pointName.trim() || !reference.trim()}
        >
          Guardar Punto de Control
        </button>
      </form>

      <div style={{ marginTop: '3rem' }}>
        <h3 style={{ fontSize: '1.2rem', color: 'var(--text-secondary)', marginBottom: '1rem' }}>
          Equipos Registrados
        </h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
          {umas.map(u => (
            <div key={u.id} style={{ padding: '1rem', background: 'rgba(0,0,0,0.2)', borderRadius: '8px' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
                <strong style={{ color: 'var(--primary)' }}>{u.name}</strong>
                <button className={styles.deleteBtn} onClick={() => { if(window.confirm(`¿Borrar equipo entero?`)) removeUma(u.id); }}>Eliminar Equipo</button>
              </div>
              <ul style={{ listStyle: 'none', display: 'flex', flexDirection: 'column', gap: '0.5rem', marginLeft: '1rem' }}>
                {u.points.map(p => (
                  <li key={p.id} style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.9rem' }}>
                    <span>- {p.name} <small style={{color:'var(--text-secondary)'}}>({p.control_type})</small></span>
                    <button className={styles.deleteBtn} onClick={() => { if(window.confirm(`¿Borrar punto?`)) removePoint(u.id, p.id); }}>X</button>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default Settings;
