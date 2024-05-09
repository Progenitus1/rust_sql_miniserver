import { FC, useRef, useState, KeyboardEvent } from 'react';
import { DBTable } from '../components/DBTable';
import { useDBQuery } from '../hooks/useDBQuery';
import { DBMessage } from '../components/DBMessage';
import { RespStatus } from '../types';
import Editor from 'react-simple-code-editor';
import { highlight  } from 'prismjs';
import { SQLGrammar } from '../utils/sqlGrammar';

interface IProps {};

export const DBMainLayout: FC<IProps> = () => {

    const oldQueries = useRef<string[]>([]);
    const actualHistoryIndex = useRef<number>(0);
    const [query, setQuery] = useState<string>('');
    const { mutate: evalQuery, data: dbResult, status } = useDBQuery();

    const evaluateQuery = () => {
        evalQuery(query);
        oldQueries.current.push(query);
        actualHistoryIndex.current = oldQueries.current.length;
        setQuery('');
    }

    const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement> | KeyboardEvent<HTMLDivElement>) => {
        if (!oldQueries.current?.length) return ;

        if (e.key === 'ArrowUp') {
            if(actualHistoryIndex.current > 0) actualHistoryIndex.current = actualHistoryIndex.current - 1;
            setQuery(oldQueries.current[actualHistoryIndex.current]);
        }
        if (e.key === 'ArrowDown') {
            if(actualHistoryIndex.current < oldQueries.current.length - 1) {
                actualHistoryIndex.current = actualHistoryIndex.current + 1;
                setQuery(oldQueries.current[actualHistoryIndex.current]);
            } else {
                setQuery('');
            }
        }
    }

    return <div className="flex flex-col justify-start items-center h-full max-h-full overflow-hidden px-24 py-5">
        <h1 className="basis-auto shrink-0 text-4xl mb-10">SQL Miniserver Playground</h1>

        <div className="basis-auto shrink-0 w-full flex gap-x-5 gap-y-2 flex-wrap mb-3">
            <Editor
                onKeyDown={(e) => handleKeyDown(e)}
                className="query-input grow"
                value={query}
                onValueChange={code => setQuery(code)}
                highlight={code => highlight(code, SQLGrammar, 'sql')}
                padding={8}
                style={{
                    fontFamily: '"Fira code", "Fira Mono", monospace',
                    fontSize: 14,
                }}
                textareaClassName="query-input__textarea"
                preClassName="query-input__pre"
            />
            <button className="query-btn shrink-0 basis-auto" disabled={query === ''} onClick={() => evaluateQuery()}>Eval</button>
            { status === 'success' ? <DBMessage dbResp={dbResult} /> : null }
        </div>
        
        <div className="grow overflow-hidden w-full bg-gray">
            { status === 'success' && dbResult.status === RespStatus.Ok && dbResult.data  ? <DBTable dbTable={dbResult?.data} /> : null }
        </div>
    </div>
}