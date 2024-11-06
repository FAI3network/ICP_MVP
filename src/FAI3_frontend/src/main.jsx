import React from 'react';
import ReactDOM from 'react-dom/client';
import './index.scss';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { Leaderboard } from './pages';
import { Layout } from './components';

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <Router>
      <Routes>
        <Route path="/" element={<Layout />} >
          <Route index element={<Leaderboard />} />
        </Route>
      </Routes>
    </Router>
  </React.StrictMode>,
);
