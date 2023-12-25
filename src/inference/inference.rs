use serde_json::Value;
use tract_onnx::prelude::*;

use crate::inference::pre_process::GrayImageFloat;
use screenshots::image::{self, EncodableLayout};

type ModelType = RunnableModel<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

pub struct CRNNModel {
    model: ModelType,
    index_2_word: Vec<String>,

    pub avg_inference_time: f64,
}

impl CRNNModel {
    pub fn new(name: String, dict_name: String) -> CRNNModel {
        // let model = tract_onnx::onnx()
        //     .model_for_path(String::from("models/") + name.as_str()).unwrap()
        //     .with_input_fact(0, InferenceFact::dt_shape(f32::datum_type(), tvec!(1, 1, 32, 384))).unwrap()
        //     .into_optimized().unwrap()
        //     .into_runnable().unwrap();
        // let mut bytes = include_bytes!("../../models/model_acc100-epoch16.onnx");
        let bytes = include_bytes!("../../models/model_genshin.onnx");
        let bytes_starrail = include_bytes!("../../models/model_training_starrail.onnx");

        let model: ModelType;

        if name == "model_training_starrail.onnx" {
            model = tract_onnx::onnx()
                .model_for_read(&mut bytes_starrail.as_bytes())
                .unwrap()
                .with_input_fact(
                    0,
                    InferenceFact::dt_shape(f32::datum_type(), tvec!(1, 1, 32, 384)),
                )
                .unwrap()
                .into_optimized()
                .unwrap()
                .into_runnable()
                .unwrap();
        } else {
            model = tract_onnx::onnx()
                .model_for_read(&mut bytes.as_bytes())
                .unwrap()
                .with_input_fact(
                    0,
                    InferenceFact::dt_shape(f32::datum_type(), tvec!(1, 1, 32, 384)),
                )
                .unwrap()
                .into_optimized()
                .unwrap()
                .into_runnable()
                .unwrap();
        }

        // let content = utils::read_file_to_string(String::from("models/index_2_word.json"));
        let content = String::from(include_str!("../../models/index_2_word.json"));
        let content_starrail = String::from(include_str!("../../models/index_2_word_starrail.json"));

        let json: Value;

        if dict_name == "index_2_word_starrail.json" {
            json = serde_json::from_str(content_starrail.as_str()).unwrap();
        } else {
            json = serde_json::from_str(content.as_str()).unwrap();
        }

        let mut index_2_word: Vec<String> = Vec::new();
        let mut i = 0;
        loop {
            let word = match json.get(i.to_string()) {
                Some(x) => x,
                None => break,
            };
            index_2_word.push(word.as_str().unwrap().to_string());
            i += 1;
        }

        CRNNModel {
            model,
            index_2_word,

            avg_inference_time: 0.0,
        }
    }

    pub fn inference_string(&self, img: &GrayImageFloat) -> String {
        let tensor: Tensor =
            tract_ndarray::Array4::from_shape_fn((1, 1, 32, 384), |(_, _, y, x)| {
                img.get_pixel(x as u32, y as u32)[0]
            })
            .into();

        let result = self.model.run(tvec!(tensor)).unwrap();
        let arr = result[0].to_array_view::<f32>().unwrap();

        let shape = arr.shape();

        let mut ans = String::new();
        let mut last_word = String::new();
        for i in 0..shape[0] {
            let mut max_index = 0;
            let mut max_value = -1.0;
            for j in 0..self.index_2_word.len() {
                let value = arr[[i, 0, j]];
                if value > max_value {
                    max_value = value;
                    max_index = j;
                }
            }
            let word = &self.index_2_word[max_index];
            if *word != last_word && word != "-" {
                ans = ans + word;
            }

            last_word = word.clone();
        }

        ans
    }
}
