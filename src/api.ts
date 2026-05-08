import { invoke } from '@tauri-apps/api/core';
import type { ApiResponse, Book, Chapter, ChapterContent } from './types';

export async function fetchSearch(
  keyword: string,
  source: string
): Promise<ApiResponse<Book[]>> {
  return invoke('search_books', { keyword, source });
}

export async function fetchHotBooks(): Promise<ApiResponse<Book[]>> {
  return invoke('get_hot_books');
}

export async function fetchChapters(
  bookId: string,
  sourceKey: string
): Promise<ApiResponse<Chapter[]>> {
  return invoke('get_chapters', { bookId, source: sourceKey });
}

export async function fetchContent(
  bookId: string,
  itemId: string,
  sourceKey: string
): Promise<ApiResponse<ChapterContent>> {
  return invoke('get_chapter_content', { bookId, itemId, source: sourceKey });
}
