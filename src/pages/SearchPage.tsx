import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { fetchSearch, fetchHotBooks } from '../api';
import type { Book } from '../types';

const SOURCES = [
  { key: 'guangyu', name: '番茄 (API)' },
  { key: 'biquge900', name: '笔趣阁' },
  { key: 'qixinge', name: '七星阁' },
] as const;

export default function SearchPage() {
  const [keyword, setKeyword] = useState('');
  const [books, setBooks] = useState<Book[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [searched, setSearched] = useState(false);
  const [hotBooks, setHotBooks] = useState<Book[]>([]);
  const [hotLoading, setHotLoading] = useState(true);
  const [currentSource, setCurrentSource] = useState('guangyu');
  const navigate = useNavigate();

  useEffect(() => {
    loadHotBooks();
  }, []);

  async function loadHotBooks() {
    setHotLoading(true);
    try {
      const res = await fetchHotBooks();
      if (res.success && res.data) setHotBooks(res.data);
    } catch {
      // ignore
    } finally {
      setHotLoading(false);
    }
  }

  async function handleSearch() {
    const kw = keyword.trim();
    if (!kw) return;
    setLoading(true);
    setError('');
    setBooks([]);
    setSearched(true);
    try {
      const res = await fetchSearch(kw, currentSource);
      if (res.success && res.data) {
        setBooks(res.data);
        if (res.data.length === 0) setError('未找到相关书籍');
      } else {
        setError(res.error ?? '搜索失败');
      }
    } catch {
      setError('请求异常，请稍后重试');
    } finally {
      setLoading(false);
    }
  }

  function handleBackToHot() {
    setSearched(false);
    setBooks([]);
    setError('');
    setKeyword('');
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'Enter') handleSearch();
  }

  return (
    <div className="page search-page">
      <header className="header">
        <h1>阅读器</h1>
      </header>

      <div className="source-selector">
        {SOURCES.map((s) => (
          <button
            key={s.key}
            className={`source-tag ${currentSource === s.key ? 'active' : ''}`}
            onClick={() => setCurrentSource(s.key)}
          >
            {s.name}
          </button>
        ))}
      </div>

      <div className="search-bar">
        <input
          type="text"
          value={keyword}
          onChange={(e) => setKeyword(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="输入书名搜索..."
          autoFocus
        />
        <button onClick={handleSearch} disabled={loading}>
          {loading ? '搜索中...' : '搜索'}
        </button>
      </div>

      {loading && <div className="message loading">搜索中...</div>}
      {error && <div className="message error">{error}</div>}

      {searched && (
        <>
          <div className="section-header">
            <button className="link-btn" onClick={handleBackToHot}>
              ↩ 返回推荐
            </button>
            <span className="section-title">搜索结果</span>
          </div>
          <div className="book-list">
            {books.map((book) => (
              <div
                key={book.book_id}
                className="book-card"
                onClick={() => navigate('/chapters', { state: { book } })}
              >
                {book.cover && (
                  <img src={book.cover} alt={book.title} className="book-cover" />
                )}
                <div className="book-info">
                  <h3 className="book-title">{book.title}</h3>
                  {book.author && <span className="book-author">{book.author}</span>}
                  {book.kind && <span className="book-kind">{book.kind}</span>}
                  {book.last_chapter && (
                    <span className="book-last">最新: {book.last_chapter}</span>
                  )}
                  {book.intro && <p className="book-intro">{book.intro}</p>}
                </div>
              </div>
            ))}
          </div>
          {!loading && !error && books.length === 0 && (
            <div className="message empty">暂无结果</div>
          )}
        </>
      )}

      {!searched && (
        <>
          <h2 className="section-title hot-title">🔥 热搜榜</h2>
          {hotLoading && <div className="message loading">加载推荐中...</div>}
          <div className="hot-grid">
            {hotBooks.map((book) => (
              <div
                key={book.book_id}
                className="hot-card"
                onClick={() => navigate('/chapters', { state: { book } })}
              >
                <img
                  src={book.cover}
                  alt={book.title}
                  className="hot-cover"
                  loading="lazy"
                />
                <span className="hot-name">{book.title}</span>
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  );
}
