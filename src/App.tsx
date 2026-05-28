import { useState } from 'react';
import Dashboard from './components/Dashboard';
import Settings from './components/Settings';
import styles from './App.module.css';

function App() {
  const [activeTab, setActiveTab] = useState<'dashboard' | 'settings'>('dashboard');

  return (
    <div className={styles.container}>
      <header className={styles.header}>
        <h1>
          <span style={{ fontSize: '1.2rem' }}>⚡</span> Metasys Control
        </h1>
        <nav className={styles.nav}>
          <button 
            className={`${styles.navButton} ${activeTab === 'dashboard' ? styles.active : ''}`}
            onClick={() => setActiveTab('dashboard')}
          >
            Dashboard
          </button>
          <button 
            className={`${styles.navButton} ${activeTab === 'settings' ? styles.active : ''}`}
            onClick={() => setActiveTab('settings')}
          >
            Configuración
          </button>
        </nav>
      </header>

      <main className={styles.mainContent}>
        {activeTab === 'dashboard' ? <Dashboard /> : <Settings />}
      </main>
    </div>
  );
}

export default App;
