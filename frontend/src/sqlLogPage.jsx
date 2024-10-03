import { Button, Card, Pagination } from "flowbite-react";
import { Table, Modal } from "flowbite-react";
import { HiOutlinePlus, HiSearch, HiShoppingCart } from "react-icons/hi";
import ReactECharts from 'echarts-for-react';
import Request from "./utils/axiosUtils";
import { useEffect, useState } from "react"; import BatteryGauge from 'react-battery-gauge'
import moment from 'moment';
import 'react-toastify/dist/ReactToastify.css';
import { useNavigate, NavLink } from "react-router-dom";
import "react-datetime/css/react-datetime.css";

import { ToastContainer, toast } from 'react-toastify';
import Datetime from 'react-datetime';

const itemsPerPage = 10;

function SqlLogPage() {
    const [startDateTime, setStartDateTime] = useState(null);
    const [endDateTime, setEndDateTime] = useState(null);
    const [keyword, setKeyword] = useState('');

    const handleStartDateChange = (date) => {
        setStartDateTime(date);
    };
    const handleEndDateChange = (date) => {
        setEndDateTime(date);
    };
    const [currentPage, setCurrentPage] = useState(1);

    const onPageChange = (page) => {

        const startIdx = (page - 1) * itemsPerPage;
        const endIdx = startIdx + itemsPerPage;

        // Set the current page data using the slice of the full data
        setCurrentTableData(allTaskTableData.slice(startIdx, endIdx));
        setCurrentPage(page);
    };

    const [allTaskTableData, setAllTaskTableData] = useState([]);
    const [currentTableData, setCurrentTableData] = useState([]);
    useEffect(() => {
        getSqlLogList();
    }, []);


    const getSqlLogList = () => {
        const requestBody = {
            ...(startDateTime && { start_time: startDateTime.format('YYYY-MM-DD HH:mm:ss') }),
            ...(endDateTime && { end_time: endDateTime.format('YYYY-MM-DD HH:mm:ss') }),
            ...(keyword && { keyword }),
        };
        Request.post("/api/sqlLogs", requestBody).then((res) => {
            console.log(res);
            const mesArray = res.data.message.map(
                ({
                    id: id,
                    sync_task_id: sync_task_id,
                    query: query,
                    result: result,
                    execution_time: execution_time,
                    client_ip: client_ip,
                    sql_timestamp: sql_timestamp,
                    timestamp: timestamp

                }) => {
                    return {
                        id,
                        sync_task_id,
                        query,
                        result,
                        execution_time,
                        client_ip,
                        sql_timestamp,
                        timestamp

                    };
                }
            );
            // Calculate the start and end indices for slicing the data
            const startIndex = (currentPage - 1) * itemsPerPage;
            const endIndex = startIndex + itemsPerPage;

            // Slice the data to display only the current page items
            const currentPageData = mesArray.slice(startIndex, endIndex);
            setCurrentTableData(currentPageData);
            setAllTaskTableData(mesArray);
        });
    };

    return (
        <div className="flex flex-col">

            <div className="p-4 flex-col">
                <div className="flex flex-row gap-x-4">
                    <div className="flex justify-center items-center gap-x-1"><div>起始时间:</div>
                        <Datetime value={startDateTime}
                            inputProps={{ placeholder: '请选择起始时间' }}
                            onChange={handleStartDateChange} />
                    </div>
                    <div className="flex justify-center items-center gap-x-1"> <div>结束时间:</div>
                        <Datetime value={endDateTime}
                            inputProps={{ placeholder: '请选择起始时间' }}
                            onChange={handleEndDateChange} />
                    </div>

                    <div className="flex justify-center items-center gap-x-1"> <div>关键字:</div><input type="text" className="rounded-md" />
                    </div>

                    <Button onClick={() => { getSqlLogList() }}>查询</Button>

                </div>
                <div className="mb-4 flex justify-center">

                    <ToastContainer />


                </div>
                <div className="relative">

                    <div className="overflow-auto max-h-[800px]">
                        <Table>
                            <Table.Head>
                                <Table.HeadCell className="font-bold text-center text-xl">任务id</Table.HeadCell>
                                <Table.HeadCell className="font-bold text-center text-xl">Sql</Table.HeadCell>
                                <Table.HeadCell className="font-bold text-center text-xl">Sql结果</Table.HeadCell>

                                <Table.HeadCell className="font-bold text-center text-xl">执行时间(毫秒)</Table.HeadCell>
                                <Table.HeadCell className="font-bold text-center text-xl">机器IP</Table.HeadCell>
                                <Table.HeadCell className="font-bold text-center text-xl">sql时间戳</Table.HeadCell>

                            </Table.Head>
                            <Table.Body className="divide-y">
                                {currentTableData.map((row, index) => (
                                    <Table.Row className="bg-white dark:border-gray-700 dark:bg-gray-800" key={index}>

                                        <Table.Cell className="text-center">  {row.sync_task_id}</Table.Cell>
                                        <Table.Cell className="text-center">  {row.query}</Table.Cell>
                                        <Table.Cell className="text-center">  {row.result}</Table.Cell>

                                        <Table.Cell className="text-center">  {row.execution_time}</Table.Cell>
                                        <Table.Cell className="text-center">  {row.client_ip}</Table.Cell>
                                        <Table.Cell className="text-center">  {row.sql_timestamp}</Table.Cell>


                                    </Table.Row>
                                ))}

                            </Table.Body>
                        </Table>
                    </div>
                </div>
                <div className="sticky bottom-0 bg-white z-10">

                    <Pagination

                        layout="pagination"
                        previousLabel="前一页"
                        nextLabel="后一页"
                        showIcons
                        currentPage={currentPage} totalPages={Math.ceil(allTaskTableData.length / itemsPerPage)} onPageChange={onPageChange} />
                </div>

            </div>
        </div >
    );
}

export default SqlLogPage;