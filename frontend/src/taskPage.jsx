import { Button, Card, Pagination } from "flowbite-react";
import { Table, Modal } from "flowbite-react";
import { HiOutlinePlus, HiSearch, HiShoppingCart } from "react-icons/hi";
import ReactECharts from 'echarts-for-react';
import Request from "./utils/axiosUtils";
import { useEffect, useState } from "react"; import BatteryGauge from 'react-battery-gauge'
import moment from 'moment';
import 'react-toastify/dist/ReactToastify.css';

import { ToastContainer, toast } from 'react-toastify';

const pageSize = 5;

function TaskPage() {

    const [openModal, setOpenModal] = useState(false);
    const [taskName, setTaskName] = useState("");
    const [fromDatasourceName, setFromDatasourceName] = useState("");
    const [toDatasourceName, setToDatasourceName] = useState("");
    const [fromDatabaseName, setFromDatabaseName] = useState("");
    const [toDatabaseName, setToDatabaseName] = useState("");

    const [tableMapping, setTableMapping] = useState("");
    //0代表插入，1代表更新
    const [modalType, setModalType] = useState(0);
    const [taskTableData, setTaskTableData] = useState([]);
    const [syncTaskMap, setSyncTaskMap] = useState(new Map());
    useEffect(() => {
        getTaskList();
    }, []);
    useEffect(() => {
        getSyncStatus();

    }, [taskTableData]);
    // Function to add an entry to the Map
    const addEntry = (key, value) => {
        setSyncTaskMap(prevMap => {
            const newMap = new Map(prevMap);
            newMap.set(key, value);
            return newMap;
        });
    };

    const getSyncStatus = () => {
        for (let i = 0; i < taskTableData.length; i++) {
            Request.get("/api/syncTask/status/" + taskTableData[i].id).then((res) => {
                console.log(res);
                const statuses = res.data.message.status;

                // Map over the status entries to create an array
                const mesArray = Object.entries(statuses).map(([key, value]) => {
                    return {
                        statusType: key, // e.g., "RUNNING"
                        status: value.status,
                        ip: value.ip,
                        gtid_set: res.data.message.gtid_set
                    };
                });
                console.log(mesArray);
                addEntry(taskTableData[i].id, mesArray);

            });
        }
    }
    const getTaskList = () => {
        Request.get("/api/syncTask").then((res) => {
            console.log(res);
            const mesArray = res.data.message.map(
                ({
                    id: id,
                    task_name: taskName,
                    from_datasource_url: fromDatasourceUrl,
                    to_datasource_url: toDatasourceUrl,
                    from_database_name: fromDatabaseName,
                    to_database_name: toDatabaseName,
                    table_mapping: tableMapping,
                    timestamp: timestamp

                }) => {
                    return {
                        id,
                        taskName,
                        fromDatasourceUrl,
                        toDatasourceUrl,
                        fromDatabaseName,
                        toDatabaseName,
                        tableMapping,
                        timestamp

                    };
                }
            );
            setTaskTableData(mesArray);
        });
    };
    const deleteSyncTask = (id) => {
        Request.delete("/api/syncTask/" + id).then((res) => {
            console.log(res);
            if (res.data.resCode != 0) {
                // this.$message.error('添加错误:' + res.data.message);
                toast.error("删除任务出错!", {
                    position: "top-center"
                });
            } else {
            }
            window.location.reload();

        })
    }
    const addAsyncTask = () => {
        Request.post("/api/syncTask", {
            "task_name": taskName,
            "from_datasource_id": Number(fromDatasourceName),
            "from_database_name": fromDatabaseName,
            "to_datasource_id": Number(toDatasourceName),
            "to_database_name": toDatabaseName,
            "table_mapping": JSON.parse(tableMapping)

        }).then((res) => {

            console.log(res);
            if (res.data.resCode != 0) {
                // this.$message.error('添加错误:' + res.data.message);
                toast.error("添加任务出错!", {
                    position: "top-center"
                });
            } else {

                toast.info("添加任务成功!", {
                    position: "top-center"
                });
                window.location.reload();
            }
        })
            .catch(err => {
                console.log(err)
            });
    }
    function replacer(key, value) {
        // List of fields to exclude
        const excludedFields = ['gtid_set', 'status'];
        return excludedFields.includes(key) ? undefined : value;
    }
    const getSyncStatusById = (id) => {
        const syncStatus = syncTaskMap.get(id);
        return JSON.stringify(syncStatus, replacer);
    }
    const getGtidSetById = (id) => {
        if (!syncTaskMap.has(id)) {
            return null;
        }
        const syncStatus = syncTaskMap.get(id);
        return syncStatus[0].gtid_set;
    }
    return (
        <div className="flex flex-col">

            <div className="p-4 flex-col">
                <div className="mb-4 flex justify-center">

                    <Button onClick={() => setOpenModal(true)}>添加任务</Button>
                    <ToastContainer />

                    <Modal dismissible show={openModal} onClose={() => setOpenModal(false)} >
                        <div className="flex flex-col items-center gap-4 p-5 ">
                            <div className="flex items-center w-full">
                                <span className="mr-2 basis-1/3 text-right	">任务名称:</span>
                                <input type="text" className="basis-1/3 bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block  p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="数据源名称" required
                                    onChange={(e) => setTaskName(e.target.value)}
                                    value={taskName}
                                />
                            </div>
                            <div className="flex items-center   w-full">
                                <span className="mr-2 basis-1/3 text-right	">源数据源名称:</span>
                                <input type="text" className="basis-1/3 bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block  p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="数据源地址" required
                                    onChange={(e) => setFromDatasourceName(e.target.value)}
                                    value={fromDatasourceName}

                                />
                            </div>
                            <div className="flex items-center   w-full">
                                <span className="mr-2 basis-1/3 text-right	">源数据库名称:</span>
                                <input type="text" className="basis-1/3 bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block  p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="数据源地址" required
                                    onChange={(e) => setFromDatabaseName(e.target.value)}
                                    value={fromDatabaseName}

                                />
                            </div>
                            <div className="flex items-center   w-full">
                                <span className="mr-2 basis-1/3 text-right	">目标数据源名称:</span>
                                <input type="text" className="basis-1/3 bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block  p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="数据源地址" required
                                    onChange={(e) => setToDatasourceName(e.target.value)}
                                    value={toDatasourceName}

                                />
                            </div>
                            <div className="flex items-center   w-full">
                                <span className="mr-2 basis-1/3 text-right	">目标数据库名称:</span>
                                <input type="text" className="basis-1/3 bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block  p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="数据源地址" required
                                    onChange={(e) => setToDatabaseName(e.target.value)}
                                    value={toDatabaseName}

                                />
                            </div>
                            <div className="flex items-center   w-full">
                                <span className="mr-2 basis-1/3 text-right	">mapping关系:</span>
                                <input type="text" className="basis-1/3 bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block  p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="数据源地址" required
                                    onChange={(e) => setTableMapping(e.target.value)}
                                    value={tableMapping}

                                />
                            </div>
                            <div className="flex items-center  w-full">
                                <div className="mr-2  basis-1/3">
                                </div>
                                {modalType == 0 &&
                                    <Button className="basis-1/3" onClick={addAsyncTask}>添加</Button>}
                                {modalType == 1 &&
                                    <Button className="basis-1/3" onClick={confirmEditFurnace}>更新</Button>}
                            </div>
                        </div>

                    </Modal>
                </div>
                <Table>
                    <Table.Head>
                        <Table.HeadCell className="font-bold text-center text-xl">任务名称</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">源数据源地址:目标数据源地址</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">源数据库:目标数据库</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">状态</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">gtid_set</Table.HeadCell>

                        <Table.HeadCell className="font-bold text-center text-xl">操作</Table.HeadCell>
                    </Table.Head>
                    <Table.Body className="divide-y">
                        {taskTableData.map((row, index) => (
                            <Table.Row className="bg-white dark:border-gray-700 dark:bg-gray-800" key={index}>
                                <Table.Cell className="whitespace-nowrap font-medium text-gray-900 dark:text-white text-center">
                                    {row.taskName}
                                </Table.Cell>
                                <Table.Cell className="text-center">
                                    <p>
                                        {row.fromDatasourceUrl}<br />
                                    </p>
                                    {row.toDatasourceUrl}
                                </Table.Cell>

                                <Table.Cell className="text-center">
                                    <p>

                                        {row.fromDatabaseName}<br />
                                    </p>
                                    {row.toDatabaseName}
                                </Table.Cell>
                                <Table.Cell className="text-center">  {getSyncStatusById(row.id)}</Table.Cell>
                                <Table.Cell className="text-center">  {getGtidSetById(row.id)}</Table.Cell>

                                <Table.Cell className="text-center ">
                                    <p>
                                        <a href="#" className="font-medium text-cyan-600 hover:underline dark:text-cyan-500"
                                            onClick={() => deleteSyncTask(row.id)}>
                                            删除
                                        </a><br />
                                    </p>
                                    <a href="#" className="font-medium text-cyan-600 hover:underline dark:text-cyan-500 px-5"
                                        onClick={() => deleteSyncTask(row.id)}>
                                        查看
                                    </a>
                                </Table.Cell>

                            </Table.Row>
                        ))}

                    </Table.Body>
                </Table>
            </div>
        </div >
    );
}
export default TaskPage;