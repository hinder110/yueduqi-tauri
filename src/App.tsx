import { Routes, Route, Navigate } from 'react-router-dom';
import SearchPage from './pages/SearchPage';
import ChaptersPage from './pages/ChaptersPage';
import ReaderPage from './pages/ReaderPage';

export default function App() {
  return (
    <Routes>
      <Route path="/" element={<SearchPage />} />
      <Route path="/chapters" element={<ChaptersPage />} />
      <Route path="/reader" element={<ReaderPage />} />
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}
