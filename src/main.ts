import { invoke } from "@tauri-apps/api/core";
import { Window } from "@tauri-apps/api/window";

interface FileResult {
    name: string;
    path: string;
    is_app: boolean;
    score: number;
}

document.addEventListener('DOMContentLoaded', () => {
    const searchInput = document.getElementById('search-input') as HTMLInputElement;
    const resultsContainer = document.getElementById('results-container') as HTMLElement;
    let selectedIndex = -1;

    searchInput.addEventListener('input', async () => {
        const query = searchInput.value;
        try {
            console.log(`Searching for: "${query}"`);
            const results: FileResult[] = await invoke('search_files', { query });
            console.log("Results received:", results);
            displayResults(results);
        } catch (error) {
            console.error("Error invoking search_files:", error);
            resultsContainer.innerHTML = `<div class="empty-message">Error: ${error}</div>`;
        }
    });

    function displayResults(results: FileResult[]) {
        clearResults();
        if (results.length > 0) {
            selectedIndex = 0;
        } else {
            selectedIndex = -1;
            resultsContainer.innerHTML = '<div class="empty-message">Nenhum resultado encontrado</div>';
            return;
        }
        
        results.forEach((result, index) => {
            const resultItem = document.createElement('div');
            resultItem.classList.add('result-item');
            resultItem.textContent = result.name;
            resultItem.dataset.path = result.path;
            
            resultItem.addEventListener('click', () => {
                openFile(result.path);
            });

            if (index === selectedIndex) {
                resultItem.classList.add('selected');
            }

            resultsContainer.appendChild(resultItem);
        });
    }

    function clearResults() {
        resultsContainer.innerHTML = '';
    }

    async function openFile(path: string) {
        if (!path) return;
        try {
            await invoke('open_file', { path });
            await Window.getCurrent().hide();
        } catch (error) {
            console.error('Failed to open file:', error);
        }
    }

    document.addEventListener('keydown', async (e) => {
        const items = resultsContainer.querySelectorAll('.result-item');
        if (e.key === 'ArrowDown') {
            e.preventDefault();
            if (selectedIndex < items.length - 1) {
                updateSelection(selectedIndex + 1);
            }
        } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            if (selectedIndex > 0) {
                updateSelection(selectedIndex - 1);
            }
        } else if (e.key === 'Enter') {
            e.preventDefault();
            if (selectedIndex !== -1) {
                const selectedItem = items[selectedIndex] as HTMLElement;
                if(selectedItem) {
                    openFile(selectedItem.dataset.path!);
                }
            }
        } else if (e.key === 'Escape') {
            await Window.getCurrent().hide();
        }
    });

    function updateSelection(newIndex: number) {
        const items = resultsContainer.querySelectorAll('.result-item');
        const currentItem = items[selectedIndex];
        if (currentItem) {
            currentItem.classList.remove('selected');
        }

        const newItem = items[newIndex];
        if (newItem) {
            newItem.classList.add('selected');
            newItem.scrollIntoView({ block: 'nearest' });
        }
        
        selectedIndex = newIndex;
    }
    
    // Initial search for common apps
    (async () => {
        try {
            const results: FileResult[] = await invoke('search_files', { query: '' });
            displayResults(results);
        } catch (error) {
            console.error("Error on initial search:", error);
        }
    })();
});
