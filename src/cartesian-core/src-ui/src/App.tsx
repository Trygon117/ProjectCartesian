import { useState, useEffect } from 'react';

// Define the window interface to satisfy TypeScript
declare global {
  interface Window {
    __TAURI__: {
      event: {
        listen: <T>(event: string, handler: (event: { payload: T }) => void) => Promise<() => void>;
      };
    };
  }
}

// Simple styling to match the previous sci-fi look
const styles = {
  container: {
    backgroundColor: '#0d1117',
    color: '#58a6ff',
    fontFamily: 'monospace',
    height: '100vh',
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    justifyContent: 'center',
  },
  box: {
    border: '2px solid #58a6ff',
    padding: '40px',
    borderRadius: '8px',
    textAlign: 'center' as const,
    boxShadow: '0 0 20px rgba(88, 166, 255, 0.2)',
  },
  statusBox: {
    marginTop: '20px',
    padding: '15px',
    backgroundColor: '#161b22',
    border: '1px solid #30363d',
  },
  detected: { color: '#ff7b72', fontWeight: 'bold' },
  safe: { color: '#238636', fontWeight: 'bold' },
  error: { color: '#ff0000', fontWeight: 'bold', border: '1px solid red', padding: '10px' }
};

function App() {
  const [status, setStatus] = useState("SEARCHING...");
  const [pid, setPid] = useState("0");
  const [isDetected, setIsDetected] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const initTauri = async () => {
      // DEBUG: Check if Tauri is actually injected
      if (!window.__TAURI__) {
        console.error("Tauri API not found!");
        setError("API ERROR: window.__TAURI__ is missing. Check tauri.conf.json");
        return;
      }

      console.log("Tauri API found. Initializing listener...");

      try {
        const unlisten = await window.__TAURI__.event.listen<string>('process-update', (event) => {
          // DEBUG: Log the event payload to browser console
          console.log("Rust Event Received:", event);

          const currentPid = event.payload;

          if (currentPid !== "0") {
            setIsDetected(true);
            setPid(currentPid);
            setStatus(`DETECTED [PID: ${currentPid}]`);
          } else {
            setIsDetected(false);
            setPid("0");
            setStatus("SAFE");
          }
        });

        return () => unlisten();
      } catch (e) {
        console.error("Failed to listen to Tauri events", e);
        setError("API ERROR: Failed to register listener");
      }
    };

    const cleanupPromise = initTauri();

    return () => {
      cleanupPromise.then(cleanup => cleanup && cleanup());
    };
  }, []);

  return (
    <div style={styles.container}>
    <div style={styles.box}>
    <h1>CARTESIAN CORE</h1>
    <p>SYSTEM STATUS: <span style={styles.safe}>ONLINE</span></p>

    {error ? (
      <div style={styles.error}>{error}</div>
    ) : (
      <div style={styles.statusBox}>
      <div>TARGET: FIREFOX</div>
      <div style={isDetected ? styles.detected : styles.safe}>
      {status}
      </div>
      </div>
    )}
    </div>
    </div>
  );
}

export default App;
