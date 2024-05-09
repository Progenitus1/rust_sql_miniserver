import { FC, ReactNode } from 'react';
import { ReactComponent as ErrorIcon } from '../assets/error.svg'
import { ReactComponent as InfoIcon } from '../assets/info.svg'
import { IDBResponse, RespStatus } from '../types';

interface IProps {
    dbResp?: IDBResponse
}

export const DBMessage: FC<IProps> = ({ dbResp }) => {

    if (!dbResp) return <div className="basis-full min-h-[1.5rem] h-10"></div>

    return <div className="basis-full flex items-center gap-2 h-6 px-2">
        { dbResp.status === RespStatus.Error ? <ErrorIcon className="fill-red h-6 w-6" /> : null }
        { dbResp.status === RespStatus.Ok ? <InfoIcon className="fill-orange h-6 w-6" /> : null }
        <p className={`text-sm text-light-gray`}>
            {dbResp.message}
            <span className="text-light-gray/50 ml-2">({dbResp.duration})</span>
        </p>
    </div>
}