use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command};

use crate::blockchain::proto::block::Block;
use crate::callbacks::{common, Callback};
use crate::errors::OpResult;

use std::fs::OpenOptions;
use std::error::Error;
use csv::WriterBuilder;

/// Dumps all addresses with non-zero balance in a csv file
pub struct Balances {
    dump_folder: PathBuf,
    writer: BufWriter<File>,

    // key: txid + index
    unspents: HashMap<Vec<u8>, common::UnspentValue>,
    lost_value: u64,

    start_height: u64,
    end_height: u64,
}

fn block_reward(height: u64) -> u64 {
    let initial_reward = 50 * 100000000;

    let halving_interval = 210000;

    let halvings = height / halving_interval;

    let reward = initial_reward >> halvings;

    reward
}

fn write_to_csv(block_height: u64, b_reward: i64, in_v: i64, out_v: i64, lost: i64) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("output.csv")?;

    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(file);
    wtr.write_record(&[block_height.to_string(), b_reward.to_string(), in_v.to_string(), out_v.to_string(), lost.to_string()])?;
    wtr.flush()?;

    Ok(())
}

impl Balances {
    fn create_writer(cap: usize, path: PathBuf) -> OpResult<BufWriter<File>> {
        Ok(BufWriter::with_capacity(cap, File::create(path)?))
    }
}

impl Callback for Balances {
    fn build_subcommand() -> Command
    where
        Self: Sized,
    {
        Command::new("balances")
            .about("Dumps all addresses with non-zero balance to CSV file")
            .version("0.1")
            .author("gcarq <egger.m@protonmail.com>")
            .arg(
                Arg::new("dump-folder")
                    .help("Folder to store csv file")
                    .index(1)
                    .required(true),
            )
    }

    fn new(matches: &ArgMatches) -> OpResult<Self>
    where
        Self: Sized,
    {
        let dump_folder = &PathBuf::from(matches.get_one::<String>("dump-folder").unwrap());
        let cb = Balances {
            dump_folder: PathBuf::from(dump_folder),
            writer: Balances::create_writer(4000000, dump_folder.join("balances.csv.tmp"))?,
            unspents: HashMap::with_capacity(10000000),
            start_height: 0,
            end_height: 0,
            lost_value: 0,
        };
        Ok(cb)
    }

    fn on_start(&mut self, block_height: u64) -> OpResult<()> {
        self.start_height = block_height;
        info!(target: "callback", "Executing balances with dump folder: {} ...", &self.dump_folder.display());
        Ok(())
    }

    /// For each transaction in the block
    ///   1. apply input transactions (remove (TxID == prevTxIDOut and prevOutID == spentOutID))
    ///   2. apply output transactions (add (TxID + curOutID -> HashMapVal))
    /// For each address, retain:
    ///   * block height as "last modified"
    ///   * output_val
    ///   * address
    fn on_block(&mut self, block: &Block, block_height: u64) -> OpResult<()> {
        let mut in_v: i64 = 0;
        let mut out_v: i64 = 0;
        let b_reward: i64 = block_reward(block_height) as i64;
        for tx in &block.txs {
            let (_in_count, spent_value) = common::remove_unspents(tx, &mut self.unspents);
            let (_count, new_value) = common::insert_unspents(tx, block_height, &mut self.unspents);
            in_v += spent_value as i64;
            out_v += new_value as i64;
        }
        let lost = b_reward + in_v - out_v;
        if lost > 0 {
            println!("block {} b_reward {} in_v {} out_v {} lost {}", block_height, b_reward, in_v, out_v, lost);
            if let Err(err) = write_to_csv(block_height, b_reward, in_v, out_v, lost) {
                eprintln!("Failed to write to CSV: {}", err);
            }
        }

        self.lost_value += lost as u64; // 如果 self.lost_value 仍然是 u64 类型
        Ok(())
    }




    fn on_complete(&mut self, block_height: u64) -> OpResult<()> {
        self.end_height = block_height;

        self.writer
            .write_all(format!("{};{}\n", "address", "balance").as_bytes())?;

        // Collect balances for each address
        let mut balances: HashMap<&str, u64> = HashMap::new();
        for unspent in self.unspents.values() {
            let entry = balances.entry(&unspent.address).or_insert(0);
            *entry += unspent.value
        }

        for (address, balance) in balances.iter() {
            self.writer
                .write_all(format!("{};{}\n", address, balance).as_bytes())?;
        }

        fs::rename(
            self.dump_folder.as_path().join("balances.csv.tmp"),
            self.dump_folder.as_path().join(format!(
                "balances-{}-{}.csv",
                self.start_height, self.end_height
            )),
        )
        .expect("Unable to rename tmp file!");

        info!(target: "callback", "Done.\nDumped {} addresses.", balances.len());
        println!("lost_value: {}",self.lost_value);
        Ok(())
    }
}
